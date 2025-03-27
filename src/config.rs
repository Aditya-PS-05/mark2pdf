use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub pdf_config: PdfConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfConfig {
    pub page_width: f32,  // in millimeters
    pub page_height: f32, // in millimeters
    pub font_size: f32,   // in points
    pub margin: f32,      // in millimeters
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            page_width: 210.0,  // A4 width
            page_height: 297.0, // A4 height
            font_size: 12.0,
            margin: 20.0,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            input_file: PathBuf::new(),
            output_file: PathBuf::new(),
            pdf_config: PdfConfig::default(),
        }
    }

    pub fn with_input_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.input_file = path.as_ref().to_path_buf();
        self
    }

    pub fn with_output_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output_file = path.as_ref().to_path_buf();
        self
    }

    pub fn with_pdf_config(mut self, config: PdfConfig) -> Self {
        self.pdf_config = config;
        self
    }
} 