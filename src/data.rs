use anyhow;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Error as RusqliteError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Participant {
    pub name: String,
    pub email: String,
    pub extra_details: String,
    pub id: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Currency {
    pub amount: f64,
    pub currency: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SecretSanta {
    pub name: String,
    pub admin_name: String,
    pub admin_email: String,
    pub gift_date: DateTime<Utc>,
    pub max_price: Currency,
    pub msg_notes: String,
    pub participants: Vec<Participant>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateRequest {
    pub secret_santa: SecretSanta,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateResponse {
    pub game_id: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BeginRequest {
    pub game_id: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BeginResponse {
    pub ok: bool,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new() -> anyhow::Result<Db> {
        Ok(Db {
            conn: Connection::open("./db.sqlite")?,
        })
    }

    pub fn setup(&self) -> anyhow::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS Game (
                    name                TEXT NOT NULL,
                    admin_name          TEXT NOT NULL,
                    admin_email         TEXT NOT NULL,
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

    pub fn create_game(&self, game: &SecretSanta) -> anyhow::Result<i64> {
        self.conn.execute(
            "INSERT INTO Game (
                name, admin_name, admin_email, gift_date, max_price_val,
                max_price_currency, msg_notes, begun
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, 0
            )",
            params![
                game.name,
                game.admin_name,
                game.admin_email,
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

    fn get_participants(&self, game_id: i64) -> anyhow::Result<Vec<Participant>> {
        let mut stmt = self.conn.prepare(
            "SELECT ROWID, name, email, extra_details
                    FROM Participant WHERE game_id=?1",
        )?;
        let participant_list = stmt
            .query_map(params![game_id], |row| {
                Ok(Participant {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    email: row.get(2)?,
                    extra_details: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, RusqliteError>>()?;
        Ok(participant_list)
    }

    pub fn get_game(&self, game_id: i64) -> anyhow::Result<SecretSanta> {
        let mut game = self.conn.query_row(
            "SELECT
                name, admin_name, admin_email, gift_date, max_price_val,
                max_price_currency, msg_notes
             FROM Game WHERE ROWID = ?1",
            params![game_id],
            |row| {
                Ok(SecretSanta {
                    name: row.get(0)?,
                    admin_name: row.get(1)?,
                    admin_email: row.get(2)?,
                    gift_date: DateTime::parse_from_rfc3339(&row.get::<usize, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    max_price: Currency {
                        amount: row.get::<usize, f64>(4)?,
                        currency: row.get(5)?,
                    },
                    msg_notes: row.get(6)?,
                    participants: vec![],
                })
            },
        )?;

        game.participants = self.get_participants(game_id)?;
        Ok(game)
    }

    pub fn assign_and_begin(
        &self,
        game_id: i64,
        pid_maps: &HashMap<i64, i64>,
    ) -> anyhow::Result<()> {
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
