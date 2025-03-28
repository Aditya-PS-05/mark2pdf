use mark2pdf::convert_markdown_to_pdf;
use std::path::Path;

fn main() {
    let input_path = Path::new("examples/input.md");
    let output_path = Path::new("output.pdf");

    if let Err(e) = convert_markdown_to_pdf(input_path, output_path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    println!("Successfully converted {} to {}", input_path.display(), output_path.display());
} 