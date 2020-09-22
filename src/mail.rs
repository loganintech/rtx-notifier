use crate::error::NotifyError;
use crate::provider::ProviderType;
use crate::Notifier;

use chrono::Local;

use std::collections::HashSet;

pub async fn get_providers_from_mail(
    notifier: &mut Notifier,
) -> Result<HashSet<ProviderType>, NotifyError> {
    let mailbox = notifier
        .imap
        .select("INBOX")
        .map_err(|_| NotifyError::MailboxLoad)?;

    let selected = (mailbox.exists - 10..mailbox.exists)
        .map(|n| n.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let messages = notifier
        .imap
        .fetch(
            selected,
            "(ENVELOPE BODY[] FLAGS INTERNALDATE BODY[HEADER])",
        )
        .map_err(|_| NotifyError::EmailFetch)?;

    let set = messages
        .into_iter()
        .filter_map(|f| {
            let body = f.envelope()?;
            let subject = body.subject?;
            let subject = String::from_utf8(subject.to_vec()).ok()?;
            let subject = subject.to_ascii_lowercase();
            let date = f.internal_date()?;
            if subject.contains("evga") && date > notifier.config.application_config.last_seen_evga {
                notifier.config.application_config.last_seen_evga = Local::now();
                Some(ProviderType::Evga(None))
            } else if subject.contains("newegg") && date > notifier.config.application_config.last_seen_newegg {
                notifier.config.application_config.last_seen_newegg = Local::now();
                Some(ProviderType::NewEgg(None))
            } else {
                None
            }
        })
        .collect::<HashSet<ProviderType>>();

    Ok(set)
}
