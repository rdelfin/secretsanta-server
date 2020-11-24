use crate::data::{Currency, Participant};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub struct SendRequest {
    #[serde(rename = "Messages")]
    messages: Vec<Email>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Email {
    #[serde(rename = "From")]
    from: EmailAddress,
    #[serde(rename = "To")]
    to: Vec<EmailAddress>,
    #[serde(rename = "Subject")]
    subject: String,
    #[serde(rename = "TextPart")]
    text_part: String,
    #[serde(rename = "HTMLPart")]
    html_part: String,
    #[serde(rename = "CustomID")]
    custom_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailAddress {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Email")]
    email: String,
}

pub struct Mailer {
    user: String,
    password: String,
    custom_id: String,
}

impl Mailer {
    pub fn new() -> Result<Mailer> {
        Ok(Mailer {
            user: env::var("MAILJET_USER")?,
            password: env::var("MAILJET_PASSWORD")?,
            custom_id: env::var("MAILJET_CUSTOM_ID")?,
        })
    }

    fn send_email(&self, to: &str, subject: &str, text: &str, html: &str) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        let send_req = SendRequest {
            messages: vec![Email {
                from: EmailAddress {
                    name: "Secret Santa".to_string(),
                    email: "secretsanta@rdelfin.com".to_string(),
                },
                to: vec![EmailAddress {
                    name: to.to_string(),
                    email: to.to_string(),
                }],
                subject: subject.to_string(),
                text_part: text.to_string(),
                html_part: html.to_string(),
                custom_id: self.custom_id.to_string(),
            }],
        };
        println!("Json object: {}", to_string_pretty(&send_req)?);
        let res = client
            .post("https://api.mailjet.com/v3.1/send")
            .basic_auth(&self.user, Some(&self.password))
            .json::<SendRequest>(&send_req)
            .send()?;

        println!("Response: {:?}", res);
        println!("Response body: {:?}", res.text());
        Ok(())
    }

    pub fn send_admin_email(&mut self, game_id: i64, admin_email: &str) -> Result<()> {
        self.send_email(
            admin_email,
            "Welcome to this Secret Santa",
            &format!(
                "Welcome to rdelfin's Secret Santa Service! This email is to let
                you know that your game ID is {}. Please save this email since the
                ID is the only thing that will let you check details about your
                game in the future.",
                game_id
            ),
            &format!(
                "<noscript><p>Welcome to rdelfin's Secret Santa Service! This email is to let
                you know that your game ID is <b>{}<b>.</p> <p>Please save this email since the
                ID is the only thing that will let you check details about your
                game in the future.</p></noscript>",
                game_id
            ),
        )?;
        Ok(())
    }

    pub fn send_begin_email(
        &mut self,
        gifter: &Participant,
        giftee: &Participant,
        gift_date: DateTime<Utc>,
        max_price: Currency,
        game_notes: &str,
        admin_name: &str,
    ) -> Result<()> {
        self.send_email(
            &gifter.email,
            "Welcome to this Secret Santa",
            &format!(
                "Welcome {name} to this Secret Santa! This was setup by {admin}. This
                email contains information about  this Secret Santa and who you'll be
                giving a gift to so pay close attention and save it for future
                reference.

                You will be giving a gift to {giftee_name} (email: {giftee_email}). We
                have the following notes regarding your gift recepient:

                {giftee_notes}

                Now that the reveal is out of the way, here are some important details.
                {admin} has set the max gift price to {max_val} {max_currency}. The
                gift is due on {due}. {admin} also left the following message for all
                of you:

                {admin_notes}


                Enjoy and Happy Holidays!",
                name = gifter.name,
                admin = admin_name,
                giftee_name = giftee.name,
                giftee_email = giftee.email,
                giftee_notes = giftee.extra_details,
                max_val = max_price.amount,
                max_currency = max_price.currency,
                admin_notes = game_notes,
                due = gift_date.date(),
            ),
            &format!(
                "<noscript>
                <p>Welcome {name} to this Secret Santa! This was setup by {admin}. This
                email contains information about  this Secret Santa and who you'll be
                giving a gift to so pay close attention and save it for future
                reference.</p>
                <p>You will be giving a gift to {giftee_name} (email: {giftee_email}). We
                have the following notes regarding your gift recepient:</p>
                <p>{giftee_notes}</p>
                <p>Now that the reveal is out of the way, here are some important details.
                {admin} has set the max gift price to {max_val} {max_currency}. The
                gift is due on {due}. {admin} also left the following message for all
                of you:</p>
                <p>{admin_notes}</p>
                <p>Enjoy and Happy Holidays!</p>
                </noscript>",
                name = gifter.name,
                admin = admin_name,
                giftee_name = giftee.name,
                giftee_email = giftee.email,
                giftee_notes = giftee.extra_details,
                max_val = max_price.amount,
                max_currency = max_price.currency,
                admin_notes = game_notes,
                due = gift_date.date(),
            ),
        )?;
        Ok(())
    }
}
