mod config;
mod core;
mod error;

pub use crate::config::{Config, PdfConfig};
use crate::core::pdf::PdfGenerator;
use crate::error::Result;
use std::path::Path;

pub struct Mark2Pdf {
    config: Config,
    pdf_generator: PdfGenerator,
}

impl Mark2Pdf {
    pub fn new(config: Config) -> Self {
        let pdf_generator = PdfGenerator::new(config.pdf_config.clone());
        Self {
            config,
            pdf_generator,
        }
    }

    pub fn convert<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        // Read the markdown content
        let content = std::fs::read_to_string(input_path)?;
        
        // Convert markdown to HTML
        let html = self.markdown_to_html(&content)?;
        
        // Generate PDF from HTML
        self.pdf_generator.generate(&html, output_path.as_ref())
    }

    fn markdown_to_html(&self, markdown: &str) -> Result<String> {
        use pulldown_cmark::{Parser, html};
        
        let parser = Parser::new(markdown);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        
        Ok(html_output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_conversion() -> Result<()> {
        let config = Config::new();
        let converter = Mark2Pdf::new(config);
        
        let temp_dir = tempdir()?;
        let input_path = temp_dir.path().join("test.md");
        let output_path = temp_dir.path().join("test.pdf");
        
        // Write test markdown
        std::fs::write(&input_path, "# Test\n\nThis is a test.")?;
        
        // Convert
        converter.convert(&input_path, &output_path)?;
        
        // Verify output exists
        assert!(output_path.exists());
        
        // Clean up
        temp_dir.close()?;
        
        Ok(())
    }
}
