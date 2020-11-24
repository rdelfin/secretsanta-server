#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod data;

use crate::data::{BeginRequest, BeginResponse, CreateRequest, CreateResponse};
use rocket_contrib::json::Json;

#[post("/create", format = "json", data = "<req>")]
fn create(req: Json<CreateRequest>) -> Json<CreateResponse> {
    let db = data::Db::new().unwrap();

    Json(CreateResponse {
        game_id: db.create_game(&req.secret_santa).unwrap(),
    })
}

#[post("/begin", format = "json", data = "<req>")]
fn begin(req: Json<BeginRequest>) -> Json<BeginResponse> {
    Json(BeginResponse { ok: true })
}

fn main() {
    data::Db::new().unwrap().setup().unwrap();
    rocket::ignite()
        .mount("/api", routes![create, begin])
        .launch();
}
