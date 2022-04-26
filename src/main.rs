#![feature(decl_macro)]

// TODO: please clean up all code
// TODO: setup ssl
// TODO: handle all errors correctly
// TODO: website

use async_mutex::Mutex;
// use rocket::http::{Cookie, Cookies};
// use rocket::response::{status, Flash, Redirect};
use rocket::{State};
// use rocket::Data;
use config::{Config, create_new_config};
use std::fs;

mod config;
mod server;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde;

extern crate serde_xml_rs;

// #[get("/")]
// fn index() -> &'static str {
//     "Hello, world!"
// }

// #[get("/admin")]
// fn admin() -> &'static str {
//     "in admin ig"
// }

// #[post("/login", format = "plain", data = "<password>")]
// fn login(mut cookies: Cookies, password: Data) -> status::Accepted<&'static str> {
//     if std::str::from_utf8(password.peek()).unwrap() == "minecraftPass" {
//         cookies.add_private(Cookie::new("loggedIn", "true"));
//         status::Accepted(Some("LoggedIn"))
//     } else {
//         status::Accepted(Some("this iS UNACCEPTABLE"))
//     }
// }

// #[post("/logout")]
// fn logout(mut cookies: Cookies) -> Flash<Redirect> {
//     cookies.remove_private(Cookie::named("loggedIn"));
//     Flash::success(Redirect::to("/"), "Logged out.")
// }

#[post("/start")]
// fn start(mut _cookies: Cookies, manager: State<Mutex<server::Manager>>) {
async fn start(manager: &State<Mutex<server::Manager>>) {
  // if cookies.get_private("loggedIn").unwrap().value() == "true" {
  let _ = manager.lock().await.start();
  // }
}

#[post("/stop")]
// fn stop(mut _cookies: Cookies, manager: State<Mutex<server::Manager>>) {
async fn stop(manager: &State<Mutex<server::Manager>>) {
  // if cookies.get_private("loggedIn").unwrap().value() == "true" {
  let _ = manager.lock().await.stop();
  // }
}

#[post("/update")]
async fn update(manager: &State<Mutex<server::Manager>>) {
  let _ = manager.lock().await.update();
}

#[post("/delete")]
async fn delete(manager: &State<Mutex<server::Manager>>) {
  let _ = manager.lock().await.delete();
}

#[post("/create")]
async fn create(manager: &State<Mutex<server::Manager>>) {
  let _ = manager.lock().await.create().await;
}

#[post("/op/<name>")]
async fn op(manager: &State<Mutex<server::Manager>>, name: &str) {
  let _ = manager.lock().await.op(name);
}

#[post("/de-op/<name>")]
async fn de_op(manager: &State<Mutex<server::Manager>>, name: &str) {
  let _ = manager.lock().await.de_op(name);
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
  let config: Config = {
    match fs::read_to_string("config.xml") {
      Ok(file) => match serde_xml_rs::from_str::<Config>(&file) {
        Ok(conf) => conf,
        _ => create_new_config(),
      },
      _ => create_new_config(),
    }
  };

  let figment = rocket::Config::figment().merge(("port", config.port));
  rocket::custom(figment).mount("/", routes![start, stop, update, delete, create, op, de_op]).manage(Mutex::new(server::Manager::new(config))).launch().await
}
