use actix_web::{web, Responder};

const MENU_URL: &'static str = "https://zse.edu.pl/kantyna/";

fn get_menu() -> Result<String, ureq::Error> {
    let html = ureq::get(MENU_URL).call()?.into_string()?;

    Ok(html)
}

pub async fn scrape_menu() -> actix_web::Result<String> {
    let site_data = web::block(|| get_menu().expect("Coulnd't get site data")).await?;

    Ok(site_data)
}
