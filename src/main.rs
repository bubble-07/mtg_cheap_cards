use std::str::FromStr;
use std::env;

fn print_usage() {
    println!("Usage:");
    println!("cargo run price_rank_csv [scryfall_json_input]");
    println!("cargo run top_cards_under_price [scryfall_json_input] [num_cards] [max_price] [type_line_text]?");
    println!("The JSON file for this program should be the 'Oracle Cards' JSON from https://scryfall.com/docs/api/bulk-data");
    panic!();
}

struct Card {
    name: String,
    price: f64,
    edhrec_rank: usize,
    type_line: String,
}

fn price_rank_csv(filename: &str) {
    let cards = parse_json_file(filename);
    for card in cards {
        println!("{}, {}", card.price, card.edhrec_rank);
    }
}

fn top_cards_under_price(filename: &str, num_cards: usize, max_price: f64, type_line_contains: Option<&str>) {
    let mut cards = parse_json_file(filename);
    cards.retain(|card| {
        if card.price > max_price {
            false
        } else {
            if type_line_contains.is_some() {
                card.type_line.contains(type_line_contains.unwrap())
            } else {
                true
            }
        }
    });
    cards.sort_by_key(|card| card.edhrec_rank);
    cards.truncate(num_cards);
    for card in cards {
        println!("1 {}", card.name);
    }
}

fn parse_json(contents: json::JsonValue) -> Vec<Card> {
    let mut result = Vec::new();
    match contents {
        json::JsonValue::Array(listing) => {
            for object in listing.into_iter() {
                if let json::JsonValue::Object(mut object) = object {
                    let maybe_name = object.remove("name");
                    let maybe_type_line = object.remove("type_line");
                    let maybe_prices_object = object.remove("prices");
                    let maybe_edhrec_rank = object.remove("edhrec_rank");
                    if maybe_edhrec_rank.is_none() {
                        //If a card is unranked, it's likely not legal
                        //in commander. Just throw it out.
                        continue;
                    }
                    if maybe_name.is_none() ||
                       maybe_type_line.is_none() ||
                       maybe_prices_object.is_none() {
                        panic!("Card object has wrong formatting. Post-removal: {:?}", object);
                    }
                    let name = maybe_name.unwrap();
                    let name = name.as_str().unwrap();

                    let type_line = maybe_type_line.unwrap();
                    let type_line = type_line.as_str().unwrap();

                    let edhrec_rank = maybe_edhrec_rank.unwrap().dump();
                    let prices_object = maybe_prices_object.unwrap();
                    if let json::JsonValue::Object(mut prices_object) = prices_object {
                        let price = prices_object.remove("usd").unwrap();
                        if price.is_null() {
                            // TODO: Log these situations? Means that the
                            // card doesn't have a current list-price
                            continue;
                        }
                        let price = price.as_str().unwrap();
                        let price = f64::from_str(&price).unwrap();

                        let edhrec_rank = usize::from_str(&edhrec_rank).unwrap();

                        let card = Card {
                            type_line: type_line.to_string(),
                            name: name.to_string(),
                            edhrec_rank,
                            price,
                        };
                        result.push(card);
                    }
                }
            }
        },
        _ => {
            panic!("Expected an array of card objects");
        },
    }
    result
}

fn parse_json_file(filename: &str) -> Vec<Card> {
    match std::fs::read_to_string(filename) {
        Ok(string_contents) => {
            match json::parse(&string_contents) {
                Ok(contents) => {
                    parse_json(contents)
                },
                Err(e) => {
                    panic!("Failed to parse json: {:?}", e);
                }
            }
        },
        Err(e) => {
            panic!("Failed to read input file: {:?}", e);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage();
    }
    let mode: &str = &args[1];
    let filename: &str = &args[2];
    match mode {
        "price_rank_csv" => {
            price_rank_csv(&filename)
        },
        "top_cards_under_price" => {
            let num_cards = usize::from_str(&args[3]).unwrap();
            let max_price = f64::from_str(&args[4]).unwrap();
            let type_line_contains = if args.len() >= 6 {
                Some(args[5].as_str())
            } else {
                None
            };
            top_cards_under_price(&filename, num_cards, max_price, type_line_contains);
        },
        _ => {
            eprintln!("Invalid mode");
            print_usage();
        },
    }
}
