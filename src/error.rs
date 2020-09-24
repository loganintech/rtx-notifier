use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum NotifyError {
    // Imap Related Errors
    TlsCreation,
    ImapConnection(Box<imap::Error>),
    ImapLogin,
    MailboxLoad,
    EmailFetch,

    // Twilio Related Errors
    TwilioSend(twilio::TwilioError),
    ConfigUpdate,

    // Config Errors
    ConfigLoad(std::io::Error),
    ConfigParse(serde_json::Error),

    // Web Errors
    WebRequestFailed(reqwest::Error),
    WebClientError,
    NoProductFound,
    HTMLParseFailed,
    NoPage,

    // OS Command Errors
    CommandErr(std::io::Error),
    CommandResult(i32),
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
            NotifyError::WebClientError => write!(f, "WebClientError"),
        }
    }
}

impl Error for NotifyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}
