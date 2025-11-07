use thiserror::Error;

#[derive(Error, Debug)]
pub enum PromptError {
    #[error("Command '{command}' failed in module '{module}': {message}")]
    #[allow(dead_code)] // TODO: use when calling external binaries
    ExternalCommandFailed {
        command: String,
        module: String,
        message: String,
    },
}

pub type Result<T> = std::result::Result<T, PromptError>;
