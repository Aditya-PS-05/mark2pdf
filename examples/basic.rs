use mark2pdf::{Config, Mark2Pdf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new configuration
    let config = Config::new()
        .with_input_file("examples/input.md")
        .with_output_file("examples/output.pdf");

    // Create a new converter
    let converter = Mark2Pdf::new(config);

    // Create example markdown file
    std::fs::write(
        "examples/input.md",
        "# Example Document\n\nThis is an example markdown document.\n\n## Features\n\n- Markdown to PDF conversion\n- Custom configuration\n- Easy to use API"
    )?;

    // Convert markdown to PDF
    converter.convert("examples/input.md", "examples/output.pdf")?;

    println!("Successfully converted markdown to PDF!");
    Ok(())
} 