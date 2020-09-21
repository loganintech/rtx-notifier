use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum NotifyError {
    TlsCreation,
    ImapConnection(imap::Error),
    ImapLogin,
    MailboxLoad,
    EmailFetch,
    DBSubscriberSelect,
    SubscriberFromRows(serde_postgres::DeError),
    TwilioSend(twilio::TwilioError),
    ConfigUpdate,
    EmailSubjectParse,
    DBConnection,
    DBConfigSelect(tokio_postgres::Error),
    DBNoConfigFound,
}

impl fmt::Display for NotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotifyError::TlsCreation => write!(f, "TlsCreation")?,
            NotifyError::ImapLogin => write!(f, "ImapLogin")?,
            NotifyError::ImapConnection(e) => write!(f, "ImapConnection: {}", e)?,
            NotifyError::MailboxLoad => write!(f, "MailboxLoad")?,
            NotifyError::EmailFetch => write!(f, "EmailFetch")?,
            NotifyError::DBSubscriberSelect => write!(f, "DBSubscriberSelect")?,
            NotifyError::DBConfigSelect(e) => write!(f, "DBConfigSelect: {}", e)?,
            NotifyError::DBNoConfigFound => write!(f, "DBNoConfigFound")?,
            NotifyError::SubscriberFromRows(e) => write!(f, "SubscriberFromRows: {}", e)?,
            NotifyError::TwilioSend(e) => write!(f, "TwilioSend: {}", e)?,
            NotifyError::ConfigUpdate => write!(f, "ConfigUpdate")?,
            NotifyError::EmailSubjectParse => write!(f, "EmailSubjectParse")?,
            NotifyError::DBConnection => write!(f, "DBConnection")?,
        }

        write!(f, " Failed")
    }
}

impl Error for NotifyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}
