use crate::error::Mark2PdfError;
use crate::config::PdfConfig;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct PdfGenerator {
    config: PdfConfig,
}

impl PdfGenerator {
    pub fn new(config: PdfConfig) -> Self {
        Self { config }
    }

    pub fn generate(&self, html_content: &str, output_path: &Path) -> Result<(), Mark2PdfError> {
        // Create a new PDF document
        let (doc, page1, layer1) = PdfDocument::new(
            "Markdown Document",
            Mm(self.config.page_width),
            Mm(self.config.page_height),
            "Layer 1"
        );

        // Create a new font
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let font_size = self.config.font_size;

        // Add text to the page
        let mut current_layer = doc.get_page(page1).get_layer(layer1);
        
        // Parse HTML content and add it to the PDF
        // Note: This is a simplified version. In a real implementation,
        // you would want to properly parse the HTML and handle different elements
        let lines: Vec<&str> = html_content.lines().collect();
        let mut y_position = Mm(self.config.page_height - 20.0); // Start 20mm from top
        
        for line in lines {
            if y_position < Mm(20.0) {
                // Add new page if we're near the bottom
                let (page2, layer2) = doc.add_page(
                    Mm(self.config.page_width),
                    Mm(self.config.page_height),
                    "Layer 1"
                );
                current_layer = doc.get_page(page2).get_layer(layer2);
                y_position = Mm(self.config.page_height - 20.0);
            }

            current_layer.use_text(
                line,
                font_size,
                Mm(20.0),
                y_position,
                &font
            );
            
            y_position -= Mm(font_size + 2.0); // Add some spacing between lines
        }

        // Save the PDF
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        doc.save(&mut writer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_pdf_generation() -> Result<(), Mark2PdfError> {
        let config = PdfConfig::default();
        let generator = PdfGenerator::new(config);
        
        let test_html = "<h1>Test Document</h1><p>This is a test paragraph.</p>";
        let temp_dir = tempdir()?;
        let output_path = temp_dir.path().join("test.pdf");
        
        generator.generate(test_html, &output_path)?;
        
        // Verify the file was created
        assert!(output_path.exists());
        
        // Clean up
        temp_dir.close()?;
        
        Ok(())
    }
} 