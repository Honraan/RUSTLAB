use serde::Serialize;

/// Envelope tipado retornado por todos os comandos IPC do backend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: &'static str,
    pub message_pt: String,
    pub detail: Option<String>,
}

impl CommandError {
    pub fn new(code: &'static str, message_pt: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            code,
            message_pt: message_pt.into(),
            detail,
        }
    }

    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::new("INVALID_INPUT", msg, None)
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::new("PATH_FORBIDDEN", msg, None)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", msg, None)
    }

    pub fn consent(msg: impl Into<String>) -> Self {
        Self::new("CONSENT_REQUIRED", msg, None)
    }

    pub fn busy(msg: impl Into<String>) -> Self {
        Self::new("PROCESS_BUSY", msg, None)
    }

    pub fn io(msg: impl Into<String>, err: impl std::fmt::Display) -> Self {
        Self::new("IO_ERROR", msg, Some(err.to_string()))
    }
}

pub type CmdResult<T> = Result<T, CommandError>;
