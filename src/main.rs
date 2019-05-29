#![feature(proc_macro_hygiene, decl_macro)]

extern crate reqwest;
#[macro_use] extern crate rocket;
use rocket_contrib::{json::Json, serve::StaticFiles};
extern crate serde;

use std::{collections::HashMap};
use serde::{Deserialize};

fn login(username: &str, password: &str) -> Option<reqwest::Client> {
    // Assign required x-www-form-urlencoded parameters
    let mut params = HashMap::new();
    params.insert("appId", "sydsvenskan.se");
    params.insert("backup_callback", "https://www.sydsvenskan.se/auth?returnTo=%2F%3Fredirected%3D1");
    params.insert("form.password", password);
    params.insert("form.username", username);
    params.insert("remember", "true");

    // Create client with cookie store
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build().unwrap();

    // Attempt login
    let res = client.post("https://account.sydsvenskan.se/security/authenticate")
        .form(&params)
        .send();

    if res.is_err() {
        return None;
    }

    // Get the profile page to verify login status
    let res = client.get("https://www.sydsvenskan.se/profil").send();
    
    match res {
        Ok(mut res) => {
            if !res.text().unwrap().contains("Logga in") {
                return Some(client);
            } else {
                return None;
            }
        }
        Err(_) => return None
    }
}

#[derive(Deserialize)]
pub struct Validate {
    username: String,
    password: String
}

#[post("/validate", data="<data>")]
fn validate(data: Json<Validate>) -> &'static str {
    match login(&data.username, &data.password) {
        Some(_) => "{ \"status\": \"ok\" }",
        None => "{ \"status\": \"err\" }"
    }
}

fn main() {
    rocket::ignite()
    .mount("/", routes![validate])
    .mount("/", StaticFiles::from("static"))
    .launch();
}