use std::path::Path;
use crate::core::markdown::MarkdownProcessor;
use crate::core::pdf::html_to_pdf;
use error::Result;

pub mod config;
pub mod core;
pub mod error;

pub struct Mark2Pdf {
    markdown_processor: MarkdownProcessor,
}

impl Mark2Pdf {
    pub fn new() -> Self {
        Self {
            markdown_processor: MarkdownProcessor::new(),
        }
    }

    pub fn convert<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        let content = std::fs::read_to_string(input_path).map_err(|e| error::Mark2PdfError::IoError(e))?;
        let html = self.markdown_processor.process_content(&content)?;
        html_to_pdf(&html, output_path.as_ref())?;
        Ok(())
    }
}

pub fn convert_markdown_to_pdf<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
    let mark2pdf = Mark2Pdf::new();
    mark2pdf.convert(input_path, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_markdown_to_pdf_conversion() -> Result<()> {
        let markdown_content = "# Test Document\n\nThis is a test.";
        let mut input_file = NamedTempFile::new().unwrap();
        input_file.write_all(markdown_content.as_bytes()).unwrap();
        let output_file = NamedTempFile::new().unwrap();

        convert_markdown_to_pdf(input_file.path(), output_file.path())?;

        assert!(output_file.path().exists());
        Ok(())
    }
}
