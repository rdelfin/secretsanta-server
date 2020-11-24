#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod data;
mod email;

use crate::{
    data::{BeginRequest, BeginResponse, CreateRequest, CreateResponse},
    email::Mailer,
};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rocket_contrib::json::Json;
use std::collections::HashMap;

#[post("/create", format = "json", data = "<req>")]
fn create(req: Json<CreateRequest>) -> Json<CreateResponse> {
    let db = data::Db::new().unwrap();
    let mut email_sender = Mailer::new().unwrap();
    let game_id = db.create_game(&req.secret_santa).unwrap();

    email_sender
        .send_admin_email(game_id, &req.secret_santa.admin_email)
        .unwrap();
    Json(CreateResponse { game_id })
}

#[post("/begin", format = "json", data = "<req>")]
fn begin(req: Json<BeginRequest>) -> Json<BeginResponse> {
    let db = data::Db::new().unwrap();
    let mut rng = thread_rng();
    // Fetch user IDs
    let mut participant_ids = db.get_participant_ids(req.game_id).unwrap();

    // Shuffle and generate mappings
    participant_ids.shuffle(&mut rng);
    let mut mappings: HashMap<i64, i64> = HashMap::new();

    for idx in 0..participant_ids.len() {
        mappings.insert(
            participant_ids[idx],
            participant_ids[(idx + 1) % participant_ids.len()],
        );
    }

    // Assign shuffled mappings to db
    db.assign_and_begin(req.game_id, &mappings).unwrap();

    // Shuffle vector
    Json(BeginResponse { ok: true })
}

fn main() {
    data::Db::new().unwrap().setup().unwrap();
    rocket::ignite()
        .mount("/api", routes![create, begin])
        .launch();
}
