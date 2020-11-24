#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod data;

#[post("/create")]
fn create() -> &'static str {
    "Hello, world!"
}

fn main() {
    data::Db::new().unwrap().setup().unwrap();
    rocket::ignite().mount("/api", routes![create]).launch();
}
