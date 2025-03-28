use crate::error::{Mark2PdfError, Result};
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TextFormat {
    pub font_size: f32,
    pub color: (f32, f32, f32),
    pub background_color: Option<(f32, f32, f32)>,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_underline: bool,
    pub alignment: TextAlignment,
}

#[derive(Debug, Clone, Copy)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl Default for TextFormat {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            color: (0.0, 0.0, 0.0),
            background_color: None,
            is_bold: false,
            is_italic: false,
            is_underline: false,
            alignment: TextAlignment::Left,
        }
    }
}

pub struct PdfState {
    doc: PdfDocumentReference,
    current_page: PdfPageIndex,
    current_layer: PdfLayerReference,
    current_y: f32,
    margin: f32,
    page_height: f32,
    page_width: f32,
    format_stack: Vec<TextFormat>,
}

impl PdfState {
    pub fn new() -> Self {
        let (doc, page_idx, layer_idx) = PdfDocument::new(
            "PDF Document",
            Mm(210.0),  // A4 width
            Mm(297.0),  // A4 height
            "Layer 1",
        );
        let page = doc.get_page(page_idx);
        let current_layer = page.get_layer(layer_idx);
        
        Self {
            doc,
            current_page: page_idx,
            current_layer,
            current_y: 277.0,  // Start near the top with some margin
            margin: 20.0,
            page_height: 297.0,
            page_width: 210.0,
            format_stack: vec![TextFormat {
                font_size: 11.0,  // 11pt base font size
                color: (0.067, 0.067, 0.067),  // #111111
                background_color: None,
                is_bold: false,
                is_italic: false,
                is_underline: false,
                alignment: TextAlignment::Left,
            }],
        }
    }

    fn current_format(&self) -> TextFormat {
        self.format_stack.last().unwrap().clone()
    }

    fn push_format(&mut self, format: TextFormat) {
        self.format_stack.push(format);
    }

    fn pop_format(&mut self) {
        if self.format_stack.len() > 1 {
            self.format_stack.pop();
        }
    }

    fn write_text(&mut self, text: &str) -> Result<()> {
        let format = self.current_format();
        let font = if format.is_bold && format.is_italic {
            self.doc.add_builtin_font(BuiltinFont::HelveticaBoldOblique)?
        } else if format.is_bold {
            self.doc.add_builtin_font(BuiltinFont::HelveticaBold)?
        } else if format.is_italic {
            self.doc.add_builtin_font(BuiltinFont::HelveticaOblique)?
        } else {
            self.doc.add_builtin_font(BuiltinFont::Helvetica)?
        };

        // Calculate text width (approximate)
        let text_width = text.len() as f32 * format.font_size * 0.5;
        let x = match format.alignment {
            TextAlignment::Left => self.margin,
            TextAlignment::Center => (self.page_width - text_width) / 2.0,
            TextAlignment::Right => self.page_width - self.margin - text_width,
        };

        // Draw background if specified
        if let Some(bg_color) = format.background_color {
            let points = vec![
                (Point::new(Mm(x - 2.0), Mm(self.current_y + 4.0)), false),
                (Point::new(Mm(x + text_width + 2.0), Mm(self.current_y + 4.0)), false),
                (Point::new(Mm(x + text_width + 2.0), Mm(self.current_y - format.font_size * 0.35)), false),
                (Point::new(Mm(x - 2.0), Mm(self.current_y - format.font_size * 0.35)), false),
            ];
            let line = Line {
                points,
                is_closed: true,
            };
            self.current_layer.set_fill_color(Color::Rgb(Rgb::new(bg_color.0, bg_color.1, bg_color.2, None)));
            self.current_layer.add_line(line);
        }

        // Set text color and draw text
        self.current_layer.set_fill_color(Color::Rgb(Rgb::new(
            format.color.0,
            format.color.1,
            format.color.2,
            None,
        )));
        self.current_layer.use_text(text, format.font_size, Mm(x), Mm(self.current_y), &font);

        // Draw underline if needed
        if format.is_underline {
            let line = Line {
                points: vec![
                    (Point::new(Mm(x), Mm(self.current_y - 1.0)), false),
                    (Point::new(Mm(x + text_width), Mm(self.current_y - 1.0)), false),
                ],
                is_closed: false,
            };
            self.current_layer.set_outline_color(Color::Rgb(Rgb::new(
                format.color.0,
                format.color.1,
                format.color.2,
                None,
            )));
            self.current_layer.add_line(line);
        }

        Ok(())
    }

    fn add_vertical_space(&mut self, space: f32) -> Result<()> {
        self.current_y -= space;
        if self.current_y < self.margin {
            // Create new page in the same document
            let (page_idx, layer_idx) = self.doc.add_page(Mm(self.page_width), Mm(self.page_height), "Layer 1");
            let page = self.doc.get_page(page_idx);
            self.current_layer = page.get_layer(layer_idx);
            self.current_page = page_idx;
            self.current_y = self.page_height - self.margin;
        }
        Ok(())
    }

    fn add_image(&mut self, path: &str) -> Result<()> {
        let img = ::image::open(path).map_err(|e| Mark2PdfError::ImageError(e.to_string()))?;
        let dyn_img = img.to_rgba8();
        let width = dyn_img.width() as f32;
        let height = dyn_img.height() as f32;
        
        // Scale image to fit within margins while maintaining aspect ratio
        let max_width = self.page_width - 2.0 * self.margin;
        let scale = if width > max_width {
            max_width / width
        } else {
            1.0
        };
        
        let final_width = width * scale;
        let final_height = height * scale;
        
        // Center the image horizontally
        let x = (self.page_width - final_width) / 2.0;
        
        // Check if we need a new page
        if self.current_y - final_height < self.margin {
            // Create new page in the same document
            let (page_idx, layer_idx) = self.doc.add_page(Mm(self.page_width), Mm(self.page_height), "Layer 1");
            let page = self.doc.get_page(page_idx);
            self.current_layer = page.get_layer(layer_idx);
            self.current_page = page_idx;
            self.current_y = self.page_height - self.margin;
        }
        
        let image_file = ImageXObject {
            width: Px(width as usize),
            height: Px(height as usize),
            color_space: ColorSpace::Rgba,
            bits_per_component: ColorBits::Bit8,
            interpolate: true,
            image_data: dyn_img.into_raw(),
            image_filter: None,
            clipping_bbox: None,
            smask: None,
        };
        
        let image = Image::from(image_file);
        image.add_to_layer(
            self.current_layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(x)),
                translate_y: Some(Mm(self.current_y - final_height)),
                scale_x: Some(scale),
                scale_y: Some(scale),
                ..Default::default()
            },
        );
        
        self.current_y -= final_height + 10.0; // Add some space after the image
        Ok(())
    }

    fn add_horizontal_rule(&mut self) -> Result<()> {
        let line = Line {
            points: vec![
                (Point::new(Mm(self.margin), Mm(self.current_y)), false),
                (Point::new(Mm(self.page_width - self.margin), Mm(self.current_y)), false),
            ],
            is_closed: false,
        };
        self.current_layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
        self.current_layer.add_line(line);
        self.current_y -= 10.0;
        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let file = File::create(path).map_err(|e| Mark2PdfError::IoError(e))?;
        let mut writer = BufWriter::new(file);
        let doc = std::mem::replace(&mut self.doc, PdfDocument::new("New Page", Mm(self.page_width), Mm(self.page_height), "Layer 1").0);
        doc.save(&mut writer).map_err(|e| Mark2PdfError::PdfError(e.to_string()))?;
        Ok(())
    }
}

pub fn html_to_pdf(html: &str, output_path: &Path) -> Result<()> {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .unwrap();

    let mut pdf = PdfState::new();
    process_node(&dom.document, &mut pdf, true)?;
    pdf.save_to_file(output_path)?;
    Ok(())
}

fn process_node(handle: &Handle, pdf: &mut PdfState, root: bool) -> Result<()> {
    let node = handle;
    match node.data {
        NodeData::Text { ref contents } => {
            let text = contents.borrow().to_string();
            if !text.trim().is_empty() {
                pdf.write_text(&text)?;
                pdf.add_vertical_space(5.0)?;
            }
        }
        NodeData::Element { ref name, ref attrs, .. } => {
            let format = pdf.current_format();
            let mut new_format = format.clone();

            match name.local.as_ref() {
                "h1" => {
                    new_format.font_size = 24.0;
                    new_format.is_bold = true;
                    pdf.add_vertical_space(20.0)?;
                }
                "h2" => {
                    new_format.font_size = 20.0;
                    new_format.is_bold = true;
                    pdf.add_vertical_space(15.0)?;
                }
                "h3" => {
                    new_format.font_size = 16.0;
                    new_format.is_bold = true;
                    pdf.add_vertical_space(10.0)?;
                }
                "h4" => {
                    new_format.font_size = 14.0;
                    new_format.is_bold = true;
                    pdf.add_vertical_space(8.0)?;
                }
                "h5" => {
                    new_format.font_size = 11.0;
                    new_format.is_bold = true;
                    pdf.add_vertical_space(6.0)?;
                }
                "h6" => {
                    new_format.font_size = 10.0;
                    new_format.is_bold = true;
                    new_format.color = (0.4, 0.4, 0.4);  // Slightly muted color
                    pdf.add_vertical_space(6.0)?;
                }
                "strong" | "b" => {
                    new_format.is_bold = true;
                }
                "em" | "i" => {
                    new_format.is_italic = true;
                }
                "u" => {
                    new_format.is_underline = true;
                }
                "code" => {
                    new_format.font_size = 11.0;
                    new_format.background_color = Some((0.973, 0.973, 0.973));  // #f8f8f8
                }
                "pre" => {
                    new_format.font_size = 11.0;
                    new_format.background_color = Some((0.973, 0.973, 0.973));  // #f8f8f8
                    pdf.add_vertical_space(10.0)?;
                }
                "a" => {
                    new_format.color = (0.204, 0.596, 0.859);  // #3498db
                    new_format.is_underline = true;
                }
                "blockquote" => {
                    new_format.color = (0.4, 0.4, 0.4);
                    new_format.is_italic = true;
                    pdf.add_vertical_space(10.0)?;
                    
                    // Add left border
                    let line = Line {
                        points: vec![
                            (Point::new(Mm(pdf.margin + 2.0), Mm(pdf.current_y + 2.0)), false),
                            (Point::new(Mm(pdf.margin + 2.0), Mm(pdf.current_y - 20.0)), false),
                        ],
                        is_closed: false,
                    };
                    pdf.current_layer.set_outline_color(Color::Rgb(Rgb::new(0.863, 0.863, 0.863, None)));  // #dcdcdc
                    pdf.current_layer.set_outline_thickness(2.0);
                    pdf.current_layer.add_line(line);
                    
                    // Add padding for the text
                    new_format.alignment = TextAlignment::Left;
                    pdf.margin += 15.0;
                }
                "table" => {
                    pdf.add_vertical_space(10.0)?;
                }
                "tr" => {
                    // Add cell padding
                    pdf.add_vertical_space(5.0)?;
                }
                "th" => {
                    new_format.is_bold = true;
                    new_format.background_color = Some((0.973, 0.973, 0.973));  // #f8f8f8
                }
                "td" => {
                    // Add cell padding
                    pdf.margin += 5.0;
                }
                "span" => {
                    for attr in attrs.borrow().iter() {
                        if attr.name.local.as_ref() == "style" {
                            for style in attr.value.split(';') {
                                let parts: Vec<&str> = style.split(':').map(|s| s.trim()).collect();
                                if parts.len() == 2 {
                                    match parts[0] {
                                        "color" => {
                                            if let Some(color) = parse_color(parts[1]) {
                                                new_format.color = color;
                                            }
                                        }
                                        "background-color" => {
                                            if let Some(color) = parse_color(parts[1]) {
                                                new_format.background_color = Some(color);
                                            }
                                        }
                                        "text-decoration" if parts[1] == "underline" => {
                                            new_format.is_underline = true;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
                "div" => {
                    for attr in attrs.borrow().iter() {
                        if attr.name.local.as_ref() == "style" {
                            for style in attr.value.split(';') {
                                let parts: Vec<&str> = style.split(':').map(|s| s.trim()).collect();
                                if parts.len() == 2 {
                                    match parts[0] {
                                        "text-align" => {
                                            new_format.alignment = match parts[1] {
                                                "center" => TextAlignment::Center,
                                                "right" => TextAlignment::Right,
                                                _ => TextAlignment::Left,
                                            };
                                        }
                                        "color" => {
                                            if let Some(color) = parse_color(parts[1]) {
                                                new_format.color = color;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        } else if attr.name.local.as_ref() == "class" && attr.value.as_ref() == "page-break" {
                            // Add a new page
                            let (page_idx, layer_idx) = pdf.doc.add_page(Mm(pdf.page_width), Mm(pdf.page_height), "Layer 1");
                            let page = pdf.doc.get_page(page_idx);
                            pdf.current_layer = page.get_layer(layer_idx);
                            pdf.current_page = page_idx;
                            pdf.current_y = pdf.page_height - pdf.margin;
                            return Ok(());
                        }
                    }
                }
                "hr" => {
                    pdf.add_horizontal_rule()?;
                    return Ok(());
                }
                "img" => {
                    for attr in attrs.borrow().iter() {
                        if attr.name.local.as_ref() == "src" {
                            pdf.add_image(&attr.value)?;
                            return Ok(());
                        }
                    }
                }
                "p" | "br" => {
                    if !root {
                        pdf.add_vertical_space(10.0)?;
                    }
                }
                _ => {}
            }

            let old_margin = pdf.margin;
            pdf.push_format(new_format);
            for child in node.children.borrow().iter() {
                process_node(child, pdf, false)?;
            }
            pdf.pop_format();
            pdf.margin = old_margin;

            match name.local.as_ref() {
                "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "div" | "blockquote" | "table" | "tr" => {
                    pdf.add_vertical_space(10.0)?;
                }
                "pre" => {
                    pdf.add_vertical_space(10.0)?;
                }
                _ => {}
            }
        }
        _ => {
            for child in node.children.borrow().iter() {
                process_node(child, pdf, false)?;
            }
        }
    }
    Ok(())
}

fn parse_color(color_str: &str) -> Option<(f32, f32, f32)> {
    if color_str.starts_with('#') {
        let hex = &color_str[1..];
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
        } else if hex.len() == 3 {
            // Support #RGB format
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
        } else {
            None
        }
    } else {
        match color_str {
            "black" => Some((0.0, 0.0, 0.0)),
            "red" => Some((1.0, 0.0, 0.0)),
            "green" => Some((0.0, 1.0, 0.0)),
            "blue" => Some((0.0, 0.0, 1.0)),
            "white" => Some((1.0, 1.0, 1.0)),
            "gray" | "grey" => Some((0.5, 0.5, 0.5)),
            "gainsboro" => Some((0.863, 0.863, 0.863)),  // #dcdcdc
            "whitesmoke" => Some((0.961, 0.961, 0.961)), // #f5f5f5
            _ => None,
        }
    }
}