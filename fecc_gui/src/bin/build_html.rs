// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
//! Generates an HTML art guide from a Markdown file.

use pulldown_cmark::{Parser, html};
use std::fs;
use std::path::Path;

fn main() {
    println!("Generating Art Guide...");

    let input_path = "../art/ART.md";
    let output_path = "../art/index.html";

    let markdown_input =
        fs::read_to_string(input_path).unwrap_or_else(|_| panic!("Could not find {input_path}"));

    let mut description = String::new();
    let mut markdown_slice = &markdown_input[..];

    if markdown_input.starts_with("---")
        && let Some(end_offset) = markdown_input[3..].find("---")
    {
        let frontmatter = &markdown_input[3..3 + end_offset];
        markdown_slice = &markdown_input[3 + end_offset + 3..];

        for line in frontmatter.lines() {
            if let Some(stripped) = line.trim().strip_prefix("description:") {
                description = stripped.trim().to_owned();
            }
        }
    }

    let parser = Parser::new(markdown_slice);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let final_html = format!(
        r#"
<!DOCTYPE html>
<!-- Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3. -->
<html lang="en">
<head>
    <script data-website-id="f4114a98-d1b2-44eb-9929-3a308b7387dc" async src="https://analytics.introverted.social/site-metrics.js"></script>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="description" content="{description}">
    <title>FECC4e Art Guide</title>
    <link rel="stylesheet" href="../style.css">
</head>
<body>
    <main>
        {html_output}
    </main>
</body>
</html>
"#
    );

    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent).expect("Failed to create output directory");
    }

    fs::write(output_path, final_html).expect("Failed to write HTML");

    println!("Art Guide built successfully!");
}
