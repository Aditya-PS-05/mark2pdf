use crate::error::Mark2PdfError;
use crate::config::PdfConfig;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom, Node};

pub struct PdfGenerator {
    config: PdfConfig,
}

struct PdfState<'a> {
    doc: PdfDocumentReference,
    current_page: PdfPageIndex,
    current_layer: PdfLayerIndex,
    y_position: f32,
    regular_font: IndirectFontRef,
    bold_font: IndirectFontRef,
    italic_font: IndirectFontRef,
    margin: f32,
    config: &'a PdfConfig,
}

impl<'a> PdfState<'a> {
    fn add_vertical_space(&mut self, space: f32) {
        self.y_position -= space;
        if self.y_position < self.margin {
            // Add new page
            let (page, layer) = self.doc.add_page(
                Mm(self.config.page_width),
                Mm(self.config.page_height),
                "Layer 1"
            );
            self.current_page = page;
            self.current_layer = layer;
            self.y_position = self.config.page_height - self.margin;
        }
    }

    fn write_text(&mut self, text: &str, font_size: f32, style: TextStyle) {
        let font = match style {
            TextStyle::Regular => &self.regular_font,
            TextStyle::Bold => &self.bold_font,
            TextStyle::Italic => &self.italic_font,
        };

        let current_layer = self.doc.get_page(self.current_page).get_layer(self.current_layer);
        current_layer.use_text(
            text,
            font_size,
            Mm(self.margin),
            Mm(self.y_position),
            font,
        );
    }
}

#[derive(Copy, Clone)]
enum TextStyle {
    Regular,
    Bold,
    Italic,
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

        // Create fonts for different styles
        let regular_font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let bold_font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
        let italic_font = doc.add_builtin_font(BuiltinFont::HelveticaOblique)?;

        // Parse HTML
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html_content.as_bytes())
            .map_err(|e| Mark2PdfError::PdfError(format!("Failed to parse HTML: {}", e)))?;

        // Initialize PDF state
        let mut state = PdfState {
            doc,
            current_page: page1,
            current_layer: layer1,
            y_position: self.config.page_height - self.config.margin,
            regular_font,
            bold_font,
            italic_font,
            margin: self.config.margin,
            config: &self.config,
        };

        // Process the DOM tree
        self.process_node(&dom.document, &mut state, TextStyle::Regular)?;

        // Save the PDF
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        state.doc.save(&mut writer)?;

        Ok(())
    }

    fn process_node(&self, handle: &Handle, state: &mut PdfState, style: TextStyle) -> Result<(), Mark2PdfError> {
        let node: &Node = handle;
        match node.data {
            NodeData::Document => {
                // Process all child nodes
                for child in node.children.borrow().iter() {
                    self.process_node(child, state, style)?;
                }
            }
            NodeData::Element { ref name, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "h1" => {
                        state.add_vertical_space(5.0);
                        let font_size = state.config.font_size * 2.0;
                        for child in node.children.borrow().iter() {
                            let text = self.get_text_content(child);
                            if !text.is_empty() {
                                state.write_text(&text, font_size, TextStyle::Bold);
                                state.add_vertical_space(font_size * 0.2);
                            }
                        }
                    }
                    "h2" => {
                        state.add_vertical_space(4.0);
                        let font_size = state.config.font_size * 1.5;
                        for child in node.children.borrow().iter() {
                            let text = self.get_text_content(child);
                            if !text.is_empty() {
                                state.write_text(&text, font_size, TextStyle::Bold);
                                state.add_vertical_space(font_size * 0.2);
                            }
                        }
                    }
                    "p" => {
                        state.add_vertical_space(4.0);
                        for child in node.children.borrow().iter() {
                            let text = self.get_text_content(child);
                            if !text.is_empty() {
                                state.write_text(&text, state.config.font_size, TextStyle::Regular);
                                state.add_vertical_space(state.config.font_size * 0.8);
                            }
                        }
                    }
                    "ul" | "ol" => {
                        state.add_vertical_space(1.0);
                        for child in node.children.borrow().iter() {
                            self.process_node(child, state, style)?;
                        }
                        state.add_vertical_space(4.0);
                    }
                    "li" => {
                        state.add_vertical_space(3.0);
                        // Add bullet point with adjusted position
                        let text_margin = state.margin;
                        state.margin += 5.0;  // Reduced from -10.0 to -5.0 to move bullet right
                        state.write_text("•", state.config.font_size, TextStyle::Regular);
                        
                        // Move text content with more space after bullet
                        state.margin = text_margin + 10.0;  // Increased from 5.0 to 10.0 for more space after bullet
                        for child in node.children.borrow().iter() {
                            let text = self.get_text_content(child);
                            if !text.is_empty() {
                                state.write_text(&text, state.config.font_size, TextStyle::Regular);
                                state.add_vertical_space(state.config.font_size * 0.35);
                            }
                        }
                        state.margin = text_margin;  // Restore original margin
                    }
                    _ => {
                        // Process unknown elements as regular text
                        for child in node.children.borrow().iter() {
                            self.process_node(child, state, style)?;
                        }
                    }
                }
            }
            NodeData::Text { ref contents } => {
                let contents_str = contents.borrow().to_string();
                let text = contents_str.trim();
                if !text.is_empty() {
                    state.write_text(text, state.config.font_size, style);
                    state.add_vertical_space(state.config.font_size * 0.2);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn get_text_content(&self, handle: &Handle) -> String {
        let node: &Node = handle;
        match node.data {
            NodeData::Text { ref contents } => {
                let contents_str = contents.borrow().to_string();
                contents_str.trim().to_string()
            }
            _ => {
                let mut text = String::new();
                for child in node.children.borrow().iter() {
                    text.push_str(&self.get_text_content(child));
                }
                text
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pdf_generation() -> Result<(), Mark2PdfError> {
        let config = PdfConfig::default();
        let generator = PdfGenerator::new(config);
        
        let test_html = "<h1>Test Document</h1><p>This is a test paragraph.</p><ul><li>Item 1</li><li>Item 2</li></ul>";
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