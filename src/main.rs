#![feature(proc_macro_hygiene, decl_macro, iter_advance_by, option_insert)]

// TODO: please clean up all code
// TODO: setup ssl

use parking_lot::Mutex;
// use rocket::http::{Cookie, Cookies};
// use rocket::response::{status, Flash, Redirect};
use rocket::State;
// use rocket::Data;
// use serde::Deserialize;
mod config;
mod server;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde;

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
//     if std::str::from_utf8(password.peek()).unwrap() == "minecraftpass" {
//         cookies.add_private(Cookie::new("loggedin", "true"));
//         status::Accepted(Some("LoggedIn"))
//     } else {
//         status::Accepted(Some("this iS UNACCEPTABLE"))
//     }
// }

// #[post("/logout")]
// fn logout(mut cookies: Cookies) -> Flash<Redirect> {
//     cookies.remove_private(Cookie::named("loggedin"));
//     Flash::success(Redirect::to("/"), "Logged out.")
// }

#[post("/start")]
// fn start(mut _cookies: Cookies, manager: State<Mutex<server::Manager>>) {
fn start(manager: State<Mutex<server::Manager>>) {
    // if cookies.get_private("loggedin").unwrap().value() == "true" {
    let _ = manager.lock().start();
    // }
}

#[post("/stop")]
// fn stop(mut _cookies: Cookies, manager: State<Mutex<server::Manager>>) {
fn stop(manager: State<Mutex<server::Manager>>) {
    // if cookies.get_private("loggedin").unwrap().value() == "true" {
    let _ = manager.lock().stop();
    // }
}

#[post("/update")]
fn update(manager: State<Mutex<server::Manager>>) {
    let _ = manager.lock().update();
}

#[post("/delete")]
fn delete(manager: State<Mutex<server::Manager>>) {
    let _ = manager.lock().delete();
}

#[post("/create")]
fn create(manager: State<Mutex<server::Manager>>) {
    let _ = manager.lock().create();
}

fn main() {
    let config = config::Config::new();

    let cfg = rocket::config::Config::build(rocket::config::Environment::active().unwrap())
        // .address("127.0.0.1")
        .port(config.port)
        .unwrap();

    rocket::custom(cfg)
        .mount("/", routes![start, stop, update, delete, create])
        .manage(Mutex::new(server::Manager::new()))
        .launch();
}
