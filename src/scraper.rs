use std::{mem::take, vec};

use actix_web::web;
use entity::{dinner, extras, extras_dinner, menu_info, prelude::Dinner};
use scraper::{Html, Selector};
use sea_orm::{prelude::Decimal, DatabaseConnection, EntityTrait, QueryOrder, Set};
use serde::Serialize;

use crate::{convert_err_to_500, errors::ServiceError, map_db_err};

const MENU_URL: &str = "https://zse.edu.pl/kantyna/";
const TWO_PARTS_DISHES_PREFIXES: [&str; 4] = ["po ", "i ", "opiekane ", "myśliwskim"];

#[derive(Debug, Clone, Serialize, Default)]
pub struct MenuDay {
    soup: String,
    dishes: Vec<String>,
    extras: Option<String>,
}
impl MenuDay {
    fn empty() -> Self {
        Self {
            soup: String::new(),
            dishes: Vec::with_capacity(3),
            extras: None,
        }
    }
}

fn get_menu() -> Result<String, ureq::Error> {
    let html = ureq::get(MENU_URL).call()?.into_string()?;

    Ok(html)
}

//vec looks like
//idx 0: [day1, day2, day3] - soups
//idx 1: [...] - blank
//idx 2..?: [...] - dishes
//idx -1..-3: [...] - extras
fn vec_to_menu(mut vec: Vec<Vec<String>>) -> Vec<MenuDay> {
    let mut menu_days: Vec<MenuDay> = vec![MenuDay::empty(); 3];

    //soup
    let soups = &mut vec[0];
    for idx in 0..3 {
        menu_days[idx].soup = take(&mut soups[idx]);
    }

    //dishes
    for dishes in vec
        .iter_mut()
        .skip(2)
        .take_while(|dishes| !dishes.is_empty())
    {
        for idx in 0..3 {
            let curr_dish = &mut dishes[idx];
            //not all of the dishes have 3 rows
            if curr_dish.is_empty() {
                continue;
            }
            if TWO_PARTS_DISHES_PREFIXES
                .iter()
                .map(|prefix| curr_dish.starts_with(prefix))
                .any(|bl| bl)
            {
                if let Some(last_dish) = menu_days[idx].dishes.last_mut() {
                    last_dish.push(' ');
                    last_dish.push_str(curr_dish);
                }
            } else {
                menu_days[idx].dishes.push(take(curr_dish));
            }
        }
    }

    //extras
    for extras in vec.iter_mut().rev().take(3).skip(2)
    //Every dish has kompot and surówka as extras so
    //might as well just skip 'em
    {
        for idx in 0..3 {
            //extras usually is ziemniaki but can be ziemniaki / X, if so save X
            let mut extra = extras[idx].splitn(2, '/');
            if let Some(extra) = extra.nth(1) {
                menu_days[idx].extras = Some(trim_whitespace(extra));
            } else {
                menu_days[idx].extras = None;
            }
        }
    }

    menu_days
}

fn trim_whitespace(s: &str) -> String {
    let mut owned = s.trim().to_string();
    let mut prev = ' ';
    owned.retain(|ch| {
        let res = (ch != ' ' || prev != ' ') && ch != '\n';
        prev = ch;
        res
    });

    owned
}

fn remove_spaces_and_utf8(s: &str) -> String {
    s.replace(|c: char| !c.is_ascii() || c == ' ', "")
}

pub async fn scrape_menu() -> Result<Vec<(u8, MenuDay)>, ServiceError> {
    let site_data = web::block(|| get_menu().expect("Couldn't get site data"))
        .await
        .map_err(|err| convert_err_to_500(err, Some("Fetch site err")))?;

    let document = Html::parse_document(&site_data);
    let tr_selector = Selector::parse("tr[height=\"22\"]").unwrap(); //class on them changes,
                                                                     //height (hopefully) remains
    let td_selector = Selector::parse("td").unwrap();
    let dishes = document.select(&tr_selector);

    //since table is set up weirdly in HTML this is probably the best solution
    let mut mon_to_wed: Vec<_> = Vec::with_capacity(11);
    let mut thu_to_sat: Vec<_> = Vec::with_capacity(11);
    let mut is_wed = false;

    for row in dishes.take_while(|val| {
        let first = val.text().nth(1);
        first.is_some() && first.unwrap() != "PRZERWY OBIADOWE"
    }) {
        let mut vec: Vec<_> = row
            .select(&td_selector)
            .map(|td| td.text().map(trim_whitespace).collect::<String>())
            .collect();

        if vec.iter().all(|val| val.is_empty()) {
            vec.clear();
        }

        if is_wed {
            thu_to_sat.push(vec);
        } else {
            if let Some(txt) = vec.get(0) {
                if txt == "CZWARTEK" {
                    is_wed = true;
                    continue;
                }
            }
            mon_to_wed.push(vec);
        }
    }

    let mut weekly_menu: Vec<MenuDay> = Vec::with_capacity(6);
    weekly_menu.append(&mut vec_to_menu(mon_to_wed));
    weekly_menu.append(&mut vec_to_menu(thu_to_sat));

    let correct_menu = weekly_menu
        .iter_mut()
        .enumerate()
        .filter(|(_, menu)| menu.dishes != ["NIECZYNNE"] && !menu.dishes.is_empty())
        .map(|(day, menu)| (day as u8, std::mem::take(menu)))
        .collect();

    Ok(correct_menu)
}

pub async fn insert_static_extras(conn: &DatabaseConnection) -> Result<(), ServiceError> {
    let static_extras = vec![
        extras::ActiveModel {
            name: Set("ziemniaki".into()),
            price: Set(Decimal::new(10, 1)),
            image: Set("eziemniaki.jpg".into()),
            r#type: Set(entity::sea_orm_active_enums::ExtrasType::Filler),
            ..Default::default()
        },
        extras::ActiveModel {
            name: Set("surówka".into()),
            price: Set(Decimal::new(10, 1)),
            image: Set("esurowka.jpg".into()),
            r#type: Set(entity::sea_orm_active_enums::ExtrasType::Salad),
            ..Default::default()
        },
        extras::ActiveModel {
            name: Set("kompot".into()),
            price: Set(Decimal::new(5, 1)),
            image: Set("ekompot.jpg".into()),
            r#type: Set(entity::sea_orm_active_enums::ExtrasType::Beverage),
            ..Default::default()
        },
    ];
    extras::Entity::insert_many(static_extras)
        .exec(conn)
        .await
        .map_err(map_db_err)?;

    Ok(())
}

pub async fn update_menu(
    conn: &DatabaseConnection,
    mut menu: Vec<(u8, MenuDay)>,
) -> Result<(), ServiceError> {
    // insert_static_extras(conn).await?; moved to init_db
    let mut prev_last_insert_id = {
        let last_id = Dinner::find()
            .order_by(dinner::Column::Id, migration::Order::Desc)
            .one(conn)
            .await
            .map_err(map_db_err)?;
        //if there is no last user id, ther are none so table is empty
        if let Some(last_id) = last_id {
            last_id.id
        } else {
            1
        }
    };
    let mut extras_dinners_all: Vec<extras_dinner::ActiveModel> =
        Vec::with_capacity((3.5 * menu.len() as f32).round() as usize);

    for (day, menu) in menu.iter_mut() {
        let soup = dinner::ActiveModel {
            image: Set(format!("s{}.jpg", remove_spaces_and_utf8(&menu.soup))),
            name: Set(take(&mut menu.soup)),
            r#type: Set(entity::sea_orm_active_enums::Type::Soup),
            week_day: Set(*day),
            max_supply: Set(15),
            price: Set(Decimal::new(15, 1)),
            ..Default::default()
        };

        let mut dinners: Vec<_> = menu
            .dishes
            .iter_mut()
            .map(|dish| dinner::ActiveModel {
                image: Set(format!("d{}.jpg", remove_spaces_and_utf8(dish))),
                name: Set(take(dish)),
                r#type: Set(entity::sea_orm_active_enums::Type::Main),
                week_day: Set(*day),
                max_supply: Set(15),
                price: Set(Decimal::new(15, 0)),
                ..Default::default()
            })
            .collect();
        dinners.push(soup);

        let res = dinner::Entity::insert_many(dinners)
            .exec(conn)
            .await
            .map_err(map_db_err)?;

        let additional = {
            if let Some(other_extra) = menu.extras.clone() {
                let res = extras::Entity::insert(extras::ActiveModel {
                    image: Set(format!("e{}.jpg", remove_spaces_and_utf8(&other_extra))),
                    name: Set(other_extra),
                    price: Set(Decimal::new(10, 1)),
                    r#type: Set(entity::sea_orm_active_enums::ExtrasType::Filler),
                    ..Default::default()
                })
                .exec(conn)
                .await
                .map_err(map_db_err)?;
                Some(res.last_insert_id)
            } else {
                None
            }
        };

        //+2 bcs last_insert_id is weird, IDK don't ask me
        let curr_last = res.last_insert_id + 2;

        for dinner_idx in prev_last_insert_id..=curr_last {
            let mut single: Vec<_> = (1..=3)
                .map(|idx| extras_dinner::ActiveModel {
                    dinner_id: Set(dinner_idx),
                    extras_id: Set(idx),
                    ..Default::default()
                })
                .collect();
            if let Some(additional) = additional {
                single.push(extras_dinner::ActiveModel {
                    dinner_id: Set(dinner_idx),
                    extras_id: Set(additional),
                    ..Default::default()
                });
            }
            extras_dinners_all.append(&mut single);
        }

        prev_last_insert_id = curr_last + 2;
    }
    extras_dinner::Entity::insert_many(extras_dinners_all)
        .exec(conn)
        .await
        .map_err(map_db_err)?;

    //last menu update date
    let menu_info_model = menu_info::ActiveModel {
        id: Set(1),
        last_update: Set(chrono::offset::Utc::now()),
    };

    menu_info::Entity::insert(menu_info_model)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(menu_info::Column::LastUpdate)
                .update_column(menu_info::Column::LastUpdate)
                .to_owned(),
        )
        .exec(conn)
        .await
        .map_err(map_db_err)?;

    Ok(())
}
