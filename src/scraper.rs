use std::vec;

use actix_web::web;
use scraper::{Html, Selector};
use serde::Serialize;

const MENU_URL: &'static str = "https://zse.edu.pl/kantyna/";

#[derive(Debug, Clone, Serialize)]
pub struct MenuDay {
    soup: String,
    dishes: Vec<String>,
    extras: Vec<String>,
}
impl MenuDay {
    fn empty() -> Self {
        Self {
            soup: String::new(),
            dishes: Vec::with_capacity(3),
            extras: Vec::with_capacity(3),
        }
    }

    fn same_extras() -> Self {
        Self {
            soup: String::new(),
            dishes: Vec::with_capacity(3),
            extras: vec!["ziemniaki".into(), "surówka".into(), "kompot".into()],
        }
    }
}

fn get_menu() -> Result<String, ureq::Error> {
    let html = ureq::get(MENU_URL).call()?.into_string()?;

    Ok(html)
}

fn vec_to_menu(vec: &Vec<Vec<String>>) -> Vec<MenuDay> {
    let mut menu_days: Vec<MenuDay> = vec![MenuDay::same_extras(); 3];

    //soup
    let soups = &vec[0];
    for idx in 0..3 {
        menu_days[idx].soup = soups[idx].clone();
    }

    for dishes in vec.iter().skip(2).take(3) {
        for idx in 0..3 {
            menu_days[idx].dishes.push(dishes[idx].clone());
        }
    }

    menu_days
}

pub async fn scrape_menu() -> actix_web::Result<Vec<MenuDay>> {
    let site_data = web::block(|| get_menu().expect("Coulnd't get site data")).await?;

    let document = Html::parse_document(&site_data);
    let tr_selector = Selector::parse(".xl7624020").unwrap();
    let dishes = document.select(&tr_selector);

    //since table is set up weird in HTML this is probably the best solution
    let mut mon_to_wed: Vec<_> = Vec::with_capacity(11);
    let mut thu_to_sat: Vec<_> = Vec::with_capacity(11);
    let mut is_wed = false;

    for row in dishes.take_while(|val| {
        let first = val.text().nth(1);
        first.is_some() && first.unwrap() != "PRZERWY OBIADOWE"
    }) {
        let vec: Vec<_> = row
            .text()
            .skip(1)
            .map(|val| val.trim().replace("\n", ""))
            .filter(|val| !val.is_empty())
            .collect();

        if !is_wed {
            if let Some(txt) = vec.iter().nth(0) {
                if txt == "kompot" {
                    is_wed = true;
                    mon_to_wed.push(vec);
                    continue;
                }
            }
        }

        if is_wed {
            thu_to_sat.push(vec);
        } else {
            mon_to_wed.push(vec);
        }
    }
    let mut weekly_menu: Vec<MenuDay> = Vec::with_capacity(6);
    weekly_menu.append(&mut vec_to_menu(&mon_to_wed));
    weekly_menu.append(&mut vec_to_menu(&thu_to_sat));

    Ok(weekly_menu)
}
