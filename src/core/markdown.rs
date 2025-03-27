use crate::config::Config;
use crate::error::{Mark2PdfError, Result};
use pulldown_cmark::{html, Options, Parser};
use std::path::Path;

pub struct MarkdownProcessor {
    config: Config,
}

impl MarkdownProcessor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn process_file<P: AsRef<Path>>(&self, input_path: P) -> Result<String> {
        let content = std::fs::read_to_string(input_path)?;
        self.process_content(&content)
    }

    pub fn process_content(&self, content: &str) -> Result<String> {
        // Set up markdown parser options
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        // Parse markdown
        let parser = Parser::new_ext(content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Ok(html_output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_basic_markdown_conversion() {
        let config = Config::new();
        let processor = MarkdownProcessor::new(config);
        let markdown = "# Hello World\n\nThis is a test.";
        let html = processor.process_content(markdown).unwrap();
        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(html.contains("<p>This is a test.</p>"));
    }
} 