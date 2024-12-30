use thiserror::Error;

use crate::event::Envelope;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Sending render action failed")]
    ActionSendFailed(#[from] tokio::sync::mpsc::error::SendError<Envelope>),
    #[error("Error aggregation")]
    Aggregate(Vec<AppError>),
    #[error("Command execution failed")]
    ExecutionFailed(String),
    #[error("File operation failed")]
    FileOperationFailed(#[from] std::io::Error),
    #[error("Invalid mime type resolved")]
    InvalidMimeType,
    #[error("Path target is invalid")]
    InvalidTargetPath,
    #[error("Loading navigation history failed")]
    LoadHistoryFailed,
    #[error("Loading marks failed")]
    LoadMarkFailed,
    #[error("Loading quickfix failed")]
    LoadQuickFixFailed,
    #[error("Preview picker is not set")]
    PreviewPickerNotResolved,
    #[error("Generating preview protocol failed")]
    PreviewProtocolGenerationFailed,
    #[error("Loading image failed")]
    ImageOperationFailed(#[from] image::ImageError),
    #[error("Terminal not initialized")]
    TerminalNotInitialized,
    #[error("Rendering view failed")]
    ViewModelFailed(#[from] std::num::TryFromIntError),
    #[error("Watch operation on path failed")]
    WatchOperationFailed(#[from] notify::Error),
}
