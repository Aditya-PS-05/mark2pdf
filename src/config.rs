use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub input_file: Option<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub page_width: f32,
    pub page_height: f32,
    pub margin: f32,
    pub font_size: f32,
    pub enable_gfm: bool,
    pub enable_syntax_highlighting: bool,
    pub enable_math: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            input_file: None,
            output_file: None,
            page_width: 210.0, // A4 width in mm
            page_height: 297.0, // A4 height in mm
            margin: 20.0,      // Default margin in mm
            font_size: 12.0,   // Default font size in points
            enable_gfm: true,
            enable_syntax_highlighting: true,
            enable_math: false,
        }
    }

    pub fn with_input_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.input_file = Some(path.into());
        self
    }

    pub fn with_output_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.output_file = Some(path.into());
        self
    }

    pub fn with_page_size(mut self, width: f32, height: f32) -> Self {
        self.page_width = width;
        self.page_height = height;
        self
    }

    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_gfm(mut self, enable: bool) -> Self {
        self.enable_gfm = enable;
        self
    }

    pub fn with_syntax_highlighting(mut self, enable: bool) -> Self {
        self.enable_syntax_highlighting = enable;
        self
    }

    pub fn with_math(mut self, enable: bool) -> Self {
        self.enable_math = enable;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
} 