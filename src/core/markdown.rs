use crate::error::{Mark2PdfError, Result};
use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::path::Path;

pub struct MarkdownProcessor;

impl MarkdownProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn process_file<P: AsRef<Path>>(&self, input_path: P) -> Result<String> {
        let content = fs::read_to_string(input_path)
            .map_err(|e| Mark2PdfError::IoError(e))?;
        self.process_content(&content)
    }

    pub fn process_content(&self, content: &str) -> Result<String> {
        // Set up options for GitHub-flavored Markdown
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        // Parse the markdown and convert to HTML
        let parser = Parser::new_ext(content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Create the final HTML with styles
        Ok(format!(
            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
* {{ box-sizing: border-box; }}
body {{ font-family: system-ui, -apple-system, 'Segoe UI', 'Roboto', 'Helvetica Neue', sans-serif; line-height: 1.6; font-size: 11pt; color: #111; margin: 0; padding: 1em; }}
h1, h2, h3, h4, h5, h6 {{ margin: 0; padding: 0.5em 0 0.25em; line-height: 1.2; }}
h1 {{ font-size: 2em; }}
h2 {{ font-size: 1.5em; }}
h3 {{ font-size: 1.17em; }}
h4 {{ font-size: 1em; }}
h5 {{ font-size: 0.83em; }}
h6 {{ font-size: 0.67em; }}
p {{ margin: 0 0 0.75em; }}
blockquote {{ margin: 0.5em 0; padding-left: 1em; border-left: 4px solid #dcdcdc; color: #666; }}
ul, ol {{ margin: 0 0 0.75em; padding-left: 2em; }}
li {{ margin: 0.25em 0; }}
pre {{ margin: 0.75em 0; padding: 1em; background-color: #f8f8f8; border-radius: 4px; overflow-x: auto; }}
code {{ background-color: #f8f8f8; padding: 0.2em 0.4em; border-radius: 3px; font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace; font-size: 0.9em; }}
pre code {{ padding: 0; background: none; }}
img {{ max-width: 100%; height: auto; margin: 0.75em 0; }}
table {{ border-spacing: 0; border-collapse: collapse; margin: 0.75em 0; width: 100%; }}
table th, table td {{ padding: 0.5em; border: 1px solid #dcdcdc; }}
table th {{ font-weight: 600; background-color: #f8f8f8; }}
table tr:nth-child(2n) {{ background-color: #f8f8f8; }}
hr {{ margin: 1.5em 0; border: 0; border-top: 1px solid #dcdcdc; }}
.page-break {{ page-break-after: always; }}
</style>
</head>
<body>
{html_output}
</body>
</html>"#
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown_conversion() {
        let processor = MarkdownProcessor::new();
        let markdown = "# Hello World\n\nThis is a test.";
        let html = processor.process_content(markdown).unwrap();
        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(html.contains("<p>This is a test.</p>"));
    }
}