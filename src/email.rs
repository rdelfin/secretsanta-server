extern crate lettre;
extern crate lettre_email;

use crate::data::{Currency, Participant};
use anyhow::Result;
use chrono::{DateTime, Utc};
use lettre::{SmtpClient, SmtpTransport, Transport};
use lettre_email::EmailBuilder;

pub struct Mailer {
    mailer: SmtpTransport,
}

impl Mailer {
    pub fn new() -> Result<Mailer> {
        Ok(Mailer {
            mailer: SmtpClient::new_unencrypted_localhost()?.transport(),
        })
    }

    pub fn send_admin_email(&mut self, game_id: i64, admin_email: &str) -> Result<()> {
        let email = EmailBuilder::new()
            .to(admin_email)
            .from(("secretsanta@rdelfin.com", "God Knows Who honestly"))
            .subject("Welcome to this Secret Santa")
            .text(format!(
                "Welcome to rdelfin's Secret Santa Service! This email is to let
                you know that your game ID is {}. Please save this email since the
                ID is the only thing that will let you check details about your
                game in the future.",
                game_id
            ))
            .html(format!(
                "<noscript><p>Welcome to rdelfin's Secret Santa Service! This email is to let
                you know that your game ID is <b>{}<b>.</p> <p>Please save this email since the
                ID is the only thing that will let you check details about your
                game in the future.</p></noscript>",
                game_id
            ))
            .build()?;

        // Send the email
        self.mailer.send(email.into())?;
        Ok(())
    }

    pub fn send_begin_email(
        &mut self,
        gifter: &Participant,
        giftee: &Participant,
        gift_date: DateTime<Utc>,
        max_price: Currency,
        game_notes: String,
        admin_name: String,
    ) -> Result<()> {
        let email = EmailBuilder::new()
            .to((&gifter.email, &gifter.name))
            .from(("secretsanta@rdelfin.com", "God Knows Who honestly"))
            .subject("Welcome to this Secret Santa")
            .text(format!(
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
            ))
            .html(format!(
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
            ))
            .build()?;

        // Send the email
        self.mailer.send(email.into())?;
        Ok(())
    }
}
