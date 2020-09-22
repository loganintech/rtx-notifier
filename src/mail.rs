use crate::error::NotifyError;
use crate::product::Product;
use crate::Notifier;

use chrono::Local;

use std::collections::HashSet;

pub async fn get_providers_from_mail(
    notifier: &mut Notifier,
) -> Result<HashSet<Product>, NotifyError> {
    // If we have an Imap session configured
    let messages = if let Some(imap) = notifier.imap.as_mut() {
        // Select the inbox
        let mailbox = imap.select("INBOX").map_err(|_| NotifyError::MailboxLoad)?;

        // Create a sequence set of the last 50 messages. (Format 1,2,3,4,5...)
        let selected = (mailbox.exists - mailbox.exists - 50..mailbox.exists)
            .map(|n| n.to_string())
            .collect::<Vec<String>>()
            .join(",");

        // Fetch the messages from the sequence set with the properties listed. Check the IMAP RFC for more info: https://tools.ietf.org/html/rfc3501#page-54
        let messages = imap
            .fetch(
                selected,
                "(ENVELOPE BODY[] FLAGS INTERNALDATE BODY[HEADER])",
            )
            .map_err(|_| NotifyError::EmailFetch)?;

        messages
    } else {
        return Ok(HashSet::new());
    };

    // For each of the messages, check if the email contains keywords we want to see, assuming the email was received after the last previously checked one.
    let set = messages
        .into_iter()
        .filter_map(|f| {
            let body = f.envelope()?;
            let subject = body.subject?;
            let subject = String::from_utf8(subject.to_vec()).ok()?;
            let subject = subject.to_ascii_lowercase();
            let date = f.internal_date()?;
            if subject.contains("evga") && date > notifier.config.application_config.last_seen_evga
            {
                notifier.config.application_config.last_seen_evga = Local::now();
                Some(Product::Evga(None))
            } else if subject.contains("newegg")
                && date > notifier.config.application_config.last_seen_newegg
            {
                notifier.config.application_config.last_seen_newegg = Local::now();
                Some(Product::NewEgg(None))
            } else {
                None
            }
        })
        .collect::<HashSet<Product>>();

    Ok(set)
}
