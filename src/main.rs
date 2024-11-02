use reqwest::blocking::get;
use scraper::{Html, Selector};

const BASE_URL: &str = "https://tabelog.com/en/tokyo/A0000/A000000/";

// returns a list of restaurant ids from start to start + quantity
fn restaurant_ids(start: u32, quantity: u32) -> Vec<u32> {
    let mut ids = Vec::new();
    for i in start..start + quantity {
        ids.push(i);
    }
    ids
}

fn restaurant_page(url: &str) -> Html {
    let response = get(url).expect("Failed to send request");
    Html::parse_document(&response.text().expect("Failed to read response text"))
}

fn parse_restaurant_info(document: &Html) {
    // Example selectors, replace with actual selectors from the page
    let name_selector = Selector::parse(".restaurant-name").unwrap();
    let address_selector = Selector::parse(".restaurant-address").unwrap();

    if let Some(name_element) = document.select(&name_selector).next() {
        let name = name_element.text().collect::<Vec<_>>().concat();
        println!("Name: {}", name);
    }

    if let Some(address_element) = document.select(&address_selector).next() {
        let address = address_element.text().collect::<Vec<_>>().concat();
        println!("Address: {}", address);
    }
}

fn main() {
    let ids = restaurant_ids(13000001, 1);

    for id in ids {
        let url = format!("{}/{}", BASE_URL, id);
        let document = restaurant_page(&url);
        parse_restaurant_info(&document);
    }
}
