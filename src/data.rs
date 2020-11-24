use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct Participant {
    pub name: String,
    pub email: String,
    pub extra_details: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Currency {
    pub amount: f32,
    pub currency: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SecretSanta {
    pub name: String,
    pub gift_date: DateTime<Utc>,
    pub max_price: Currency,
    pub msg_notes: String,
    pub participants: Vec<Participant>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateRequest {
    pub secret_santa: SecretSanta,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateResponse {
    pub game_id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BeginRequest {
    pub game_id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BeginResponse {
    pub ok: bool,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new() -> Result<Db> {
        Ok(Db {
            conn: Connection::open("./db.sqlite")?,
        })
    }

    pub fn setup(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS Game (
                    name                TEXT NOT NULL,
                    gift_date           VARCHAR(100),
                    max_price_val       REAL,
                    max_price_currency  VARCHAR(5),
                    msg_notes           TEXT,
                    begun               BOOLEAN
             )",
            params![],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS Participant (
                    game_id        INTEGER NOT NULL,
                    name           TEXT NOT NULL,
                    email          TEXT NOT NULL,
                    extra_details  TEXT,
                    gift_to        INTEGER,
                    FOREIGN KEY(game_id) REFERENCES Game(ROWID)
                    FOREIGN KEY(gift_to) REFERENCES Participant(ROWID)
             )",
            params![],
        )?;

        Ok(())
    }

    pub fn create_game(&self, game: &SecretSanta) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO Game (
                name, gift_date, max_price_val, max_price_currency, msg_notes, begun
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, 0
            )",
            params![
                game.name,
                game.gift_date.to_rfc3339(),
                game.max_price.amount.to_string(),
                game.max_price.currency,
                game.msg_notes,
            ],
        )?;
        let game_id = self.conn.last_insert_rowid();

        for participant in &game.participants {
            self.conn.execute(
                "INSERT INTO Participant (
                    game_id, name, email, extra_details
                ) VALUES (
                    ?1, ?2, ?3, ?4
                )",
                params![
                    game_id,
                    participant.name,
                    participant.email,
                    participant.extra_details
                ],
            )?;
        }

        Ok(game_id)
    }

    pub fn get_participant_ids(&self, game_id: i64) -> Result<Vec<i64>> {
        let mut stmt = self
            .conn
            .prepare("SELECT ROWID FROM Participant WHERE game_id=?1")?;
        let row_map = stmt.query_map(params![game_id], |row| row.get(0))?;

        let mut result = vec![];
        for row in row_map {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn assign_and_begin(&self, game_id: i64, pid_maps: &HashMap<i64, i64>) -> Result<()> {
        for (gifter, giftee) in pid_maps {
            self.conn.execute(
                "UPDATE Participant SET gift_to = ?1 WHERE ROWID = ?2",
                params![giftee, gifter],
            )?;
        }

        self.conn.execute(
            "UPDATE Game SET begun = 1 WHERE ROWID = ?1",
            params![game_id],
        )?;

        Ok(())
    }
}
