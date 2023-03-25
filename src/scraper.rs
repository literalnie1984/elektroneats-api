use std::vec;

use actix_web::web;
use scraper::{Html, Selector};
use serde::Serialize;

const MENU_URL: &'static str = "https://zse.edu.pl/kantyna/";
const TWO_PARTS_DISHES_PREFIXES: [&'static str; 2] = ["po ", "i "];

#[derive(Debug, Clone, Serialize)]
pub struct MenuDay {
    soup: String,
    dishes: Vec<String>,
    extras: String,
}
impl MenuDay {
    fn empty() -> Self {
        Self {
            soup: String::new(),
            dishes: Vec::with_capacity(3),
            extras: String::new(),
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
fn vec_to_menu(vec: &Vec<Vec<String>>) -> Vec<MenuDay> {
    let mut menu_days: Vec<MenuDay> = vec![MenuDay::empty(); 3];

    //soup
    let soups = &vec[0];
    for idx in 0..3 {
        menu_days[idx].soup = soups[idx].clone();
    }

    //dishes
    for dishes in vec.iter().skip(2).take_while(|dishes| !dishes.is_empty()) {
        for idx in 0..3 {
            let curr_dish = &dishes[idx];
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
                    last_dish.push_str(" ");
                    last_dish.push_str(curr_dish);
                }
            } else {
                menu_days[idx].dishes.push(curr_dish.clone());
            }
        }
    }

    //extras
    for extras in vec.iter().rev().take(3).skip(2)
    //Every dish has kompot and surówka as extras so
    //might just skip 'em
    {
        for idx in 0..3 {
            menu_days[idx].extras = extras[idx].clone();
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

pub async fn scrape_menu() -> actix_web::Result<Vec<MenuDay>> {
    let site_data = web::block(|| get_menu().expect("Couldn't get site data")).await?;

    let document = Html::parse_document(&site_data);
    let tr_selector = Selector::parse(".xl7624020").unwrap();
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
            .into_iter()
            .map(|td| td.text().map(trim_whitespace).collect::<String>())
            .collect();

        if vec.iter().all(|val| val.is_empty()) {
            vec.clear();
        }

        if is_wed {
            thu_to_sat.push(vec);
        } else {
            if let Some(txt) = vec.iter().nth(0) {
                if txt == "kompot" {
                    is_wed = true;
                }
            }
            mon_to_wed.push(vec);
        }
    }

    let mut weekly_menu: Vec<MenuDay> = Vec::with_capacity(6);
    weekly_menu.append(&mut vec_to_menu(&mon_to_wed));
    weekly_menu.append(&mut vec_to_menu(&thu_to_sat));

    Ok(weekly_menu)
}
