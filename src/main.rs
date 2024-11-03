use reqwest::{blocking::get, Url};
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
    rating: Option<f64>,
    latitude: f64,
    longitude: f64,
}

impl Restaurant {
    fn new(
        id: u32,
        name: String,
        category: String,
        rating: Option<f64>,
        latitude: f64,
        longitude: f64,
    ) -> Self {
        Restaurant {
            id,
            name,
            category,
            rating,
            latitude,
            longitude,
        }
    }
}

#[derive(Debug)]
struct TabelogRestaurantPageData {
    name: String,
    category: String,
    rating: Option<f64>,
    latitude: f64,
    longitude: f64,
}

impl TabelogRestaurantPageData {
    fn new(
        name: String,
        category: String,
        rating: Option<f64>,
        latitude: f64,
        longitude: f64,
    ) -> Self {
        TabelogRestaurantPageData {
            name,
            category,
            rating,
            latitude,
            longitude,
        }
    }
}

use std::fs::File;
use std::io::Write;

fn parse_restaurant_info(document: &Html) -> TabelogRestaurantPageData {
    // Example selectors, replace with actual selectors from the page
    let table_selector = Selector::parse(".rstinfo-table").unwrap();
    let table = document
        .select(&table_selector)
        .next()
        .expect("Table not found");

    let name_selector = Selector::parse(".rstinfo-table__name-wrap").unwrap();
    let name = if let Some(name_element) = table.select(&name_selector).next() {
        name_element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .trim()
            .to_string()
    } else {
        String::new()
    };
    // println!("Name: {}", name);

    let rating_selector = Selector::parse(".rdheader-rating__score-val-dtl").unwrap();
    let rating: Option<f64> = if let Some(rating_element) = document.select(&rating_selector).next()
    {
        rating_element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .parse::<f64>()
            .ok()
    } else {
        None
    };
    // println!("Rating: {}", rating);

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
                category = td.text().collect::<Vec<_>>().concat().trim().to_string();
            }
            //  else if header == "Address" {
            //     let address_text = td.text().collect::<Vec<_>>().concat();
            //     let address_filtered: String = address_text
            //         .chars()
            //         .filter(|c| !c.is_whitespace())
            //         .collect();
            //     // println!(
            //     //     "{:?}",
            //     //     address_filtered.replace("ShowlargermapFindnearbyrestaurants", "")
            //     // );
            //     address = address_filtered.replace("ShowlargermapFindnearbyrestaurants", "");
            // }
        }
    }

    // println!("Category: {}", category);
    // println!("Address: {}", address);

    // Extract the data-original attribute
    let map_image_selector = Selector::parse(".rstinfo-table__map-image").unwrap();
    let map_image_url = if let Some(map_image_element) = document.select(&map_image_selector).next()
    {
        map_image_element
            .value()
            .attr("data-original")
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };

    // Parse the URL to extract the markers coordinate
    let markers_coordinate = if let Ok(parsed_url) = Url::parse(&map_image_url) {
        parsed_url
            .query_pairs()
            .find_map(|(key, value)| {
                if key == "markers" {
                    value.split('|').nth(1).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(String::new)
    } else {
        String::new()
    };

    // println!("Map Image URL: {}", map_image_url);
    let mut latitude: f64 = 0.0;
    let mut longitude: f64 = 0.0;
    if let Some(coords) = markers_coordinate
        .split(',')
        .collect::<Vec<&str>>()
        .get(0..2)
    {
        if let [lat_str, long_str] = &coords[..] {
            latitude = lat_str.parse().unwrap();
            longitude = long_str.parse().unwrap();
            // println!("Latitude: {}", latitude);
            // println!("Longitude: {}", longitude);
        }
    }

    return TabelogRestaurantPageData::new(name, category, rating, latitude, longitude);
}

use std::io::{self, BufWriter};

fn save_restaurant_to_csv(restaurant: &Restaurant, writer: &mut BufWriter<File>) -> io::Result<()> {
    let csv_line = format!(
        "{},{},{},{},{},{}\n",
        restaurant.id,
        restaurant.name,
        restaurant.category,
        restaurant
            .rating
            .map_or_else(|| "null".to_string(), |r| r.to_string()),
        restaurant.latitude.to_string(),
        restaurant.longitude.to_string()
    );

    writer.write_all(csv_line.as_bytes())
}

fn main() {
    let ids = restaurant_ids(13000001, 100);

    let file_path = "results.csv";
    let file = File::create(file_path).expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    for id in ids {
        let url = format!("{}/{}", BASE_URL, id);
        let document = restaurant_page(&url);
        let scrape_info = parse_restaurant_info(&document);
        let restaurant = Restaurant::new(
            id,
            scrape_info.name,
            scrape_info.category,
            scrape_info.rating,
            scrape_info.latitude,
            scrape_info.longitude,
        );
        let res = save_restaurant_to_csv(&restaurant, &mut writer);
        if let Err(e) = res {
            eprintln!("Error: {}", e);
        }
    }
}

// fn write_restaurant_to_csv(
//     writer: &mut BufWriter<File>,
//     restaurant: &Restaurant,
// ) -> io::Result<()> {
//     writeln!(
//         writer,
//         "{},{},{},{},{},{}",
//         restaurant.id,
//         restaurant.name,
//         restaurant.category,
//         restaurant.rating,
//         restaurant.latitude,
//         restaurant.longitude
//     )
// }
