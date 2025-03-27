use thiserror::Error;

#[derive(Error, Debug)]
pub enum Mark2PdfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("PDF generation error: {0}")]
    PdfError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Markdown processing error: {0}")]
    MarkdownError(String),
}

impl From<printpdf::Error> for Mark2PdfError {
    fn from(err: printpdf::Error) -> Self {
        Mark2PdfError::PdfError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Mark2PdfError>; 