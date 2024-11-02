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

#[derive(Debug)]
struct Restaurant {
    id: u32,
    name: String,
    category: String,
    address: String,
}

impl Restaurant {
    fn new(id: u32, name: String, category: String, address: String) -> Self {
        Restaurant {
            id,
            name,
            category,
            address,
        }
    }
}

#[derive(Debug)]
struct RestaurantParsedData {
    name: String,
    category: String,
    address: String,
}

impl RestaurantParsedData {
    fn new(name: String, category: String, address: String) -> Self {
        RestaurantParsedData {
            name,
            category,
            address,
        }
    }
}

fn parse_restaurant_info(document: &Html) -> RestaurantParsedData {
    // Example selectors, replace with actual selectors from the page
    let table_selector = Selector::parse(".rstinfo-table").unwrap();
    let table = document
        .select(&table_selector)
        .next()
        .expect("Table not found");

    let name_selector = Selector::parse(".rstinfo-table__name-wrap").unwrap();
    let name = if let Some(name_element) = table.select(&name_selector).next() {
        name_element.text().collect::<Vec<_>>().concat()
    } else {
        String::new()
    };
    println!("Name: {}", name);

    let rating_selector = Selector::parse(".rdheader-rating__score-val-dtl").unwrap();
    let rating = if let Some(rating_element) = document.select(&rating_selector).next() {
        rating_element.text().collect::<Vec<_>>().concat()
    } else {
        String::new()
    };
    println!("Rating: {}", rating);

    let mut category = String::new();
    let mut address = String::new();
    let table_rows = Selector::parse("tr").unwrap();
    for row in table.select(&table_rows) {
        let th = row.select(&Selector::parse("th").unwrap()).next();
        let td = row.select(&Selector::parse("td").unwrap()).next();
        if let (Some(th), Some(td)) = (th, td) {
            let header_text = th.text().collect::<Vec<_>>().concat();
            let header = header_text.trim();
            if header == "Categories" {
                category = td.text().collect::<Vec<_>>().concat();
            } else if header == "Address" {
                let address_text = td.text().collect::<Vec<_>>().concat();
                let address_filtered: String = address_text
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect();
                println!(
                    "{:?}",
                    address_filtered.replace("ShowlargermapFindnearbyrestaurants", "")
                );
                address = address_filtered.replace("ShowlargermapFindnearbyrestaurants", "");
            }
        }
    }

    println!("Category: {}", category);
    println!("Address: {}", address);

    return RestaurantParsedData::new(name, category, address);
}

fn main() {
    let ids = restaurant_ids(13000001, 10);

    for id in ids {
        let url = format!("{}/{}", BASE_URL, id);
        let document = restaurant_page(&url);
        parse_restaurant_info(&document);
    }
}
