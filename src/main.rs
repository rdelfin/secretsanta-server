#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod data;
mod email;

use crate::{
    data::{BeginRequest, BeginResponse, CreateRequest, CreateResponse, Participant},
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
    let mut email_sender = Mailer::new().unwrap();
    let mut rng = thread_rng();

    // Fetch users
    let game = db.get_game(req.game_id).unwrap();
    let mut participants = game.participants.clone();

    // Shuffle and generate mappings
    participants.shuffle(&mut rng);
    let mut mappings: HashMap<i64, i64> = HashMap::new();
    let mut participant_mappings: Vec<(&Participant, &Participant)> = vec![];

    for idx in 0..participants.len() {
        mappings.insert(
            participants[idx].id.unwrap(),
            participants[(idx + 1) % participants.len()].id.unwrap(),
        );
        participant_mappings.push((
            &participants[idx],
            &participants[(idx + 1) % participants.len()],
        ))
    }

    // Assign shuffled mappings to db
    db.assign_and_begin(req.game_id, &mappings).unwrap();

    for (gifter, giftee) in &participant_mappings {
        email_sender
            .send_begin_email(
                gifter,
                giftee,
                &game.gift_date,
                &game.max_price,
                &game.msg_notes,
                &game.admin_name,
            )
            .unwrap();
    }

    Json(BeginResponse { ok: true })
}

fn main() {
    data::Db::new().unwrap().setup().unwrap();
    rocket::ignite()
        .mount("/api", routes![create, begin])
        .launch();
}
