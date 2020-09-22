use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum NotifyError {
    TlsCreation,
    ImapConnection(imap::Error),
    ImapLogin,
    MailboxLoad,
    EmailFetch,
    TwilioSend(twilio::TwilioError),
    ConfigUpdate,
    // EmailSubjectParse,
    ConfigLoad(std::io::Error),
    ConfigParse(serde_json::Error),
    WebRequestFailed(reqwest::Error),
    HTMLParseFailed,
    NoProductFound,
    CommandErr(std::io::Error),
    CommandResult(i32),
    NoPage,
}

impl fmt::Display for NotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotifyError::TlsCreation => write!(f, "TlsCreation"),
            NotifyError::ImapLogin => write!(f, "ImapLogin"),
            NotifyError::ImapConnection(e) => write!(f, "ImapConnection: {}", e),
            NotifyError::MailboxLoad => write!(f, "MailboxLoad"),
            NotifyError::EmailFetch => write!(f, "EmailFetch"),
            NotifyError::ConfigLoad(e) => write!(f, "ConfigLoad: {}", e),
            NotifyError::ConfigParse(e) => write!(f, "ConfigParse: {}", e),
            NotifyError::TwilioSend(e) => write!(f, "TwilioSend: {}", e),
            NotifyError::ConfigUpdate => write!(f, "ConfigUpdate"),
            // NotifyError::EmailSubjectParse => write!(f, "EmailSubjectParse"),
            NotifyError::WebRequestFailed(e) => write!(f, "WebRequestFailed: {}", e),
            NotifyError::HTMLParseFailed => write!(f, "HTMLParseFailed"),
            NotifyError::NoProductFound => write!(f, "NoProductFound"),
            NotifyError::CommandErr(e) => write!(f, "CommandErr: {}", e),
            NotifyError::CommandResult(e) => write!(f, "CommandResult: {}", e),
            NotifyError::NoPage => write!(f, "NoPage"),
        }
    }
}

impl Error for NotifyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}
