// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Generates Cloudflare's `_headers` file for the built site.
//!
//! Runs as a Trunk hook, as the policy needs hashes of the final page's inline scripts.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest as _, Sha256};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("Generating _headers...");

    let staging_dir = env::var("TRUNK_STAGING_DIR").expect("Should be run as a Trunk hook");
    let staging_dir = Path::new(&staging_dir);

    let index_html =
        fs::read_to_string(staging_dir.join("index.html")).expect("Could not read index.html");

    let script_hashes: Vec<String> = inline_scripts(&index_html)
        .into_iter()
        .map(csp_hash)
        .collect();
    assert!(
        !script_hashes.is_empty(),
        "Could not find Trunk's inline loader script"
    );

    fs::write(
        staging_dir.join("_headers"),
        headers(&script_hashes.join(" ")),
    )
    .expect("Failed to write _headers");

    println!("_headers built successfully!");
}

/// Returns the contents of each script that the browser runs inline.
fn inline_scripts(html: &str) -> Vec<&str> {
    let mut scripts = Vec::new();
    let mut rest = html;

    while let Some((_, after_start)) = rest.split_once("<script") {
        let (attributes, after_attributes) = after_start
            .split_once('>')
            .expect("Unterminated script tag");
        let (content, after_end) = after_attributes
            .split_once("</script>")
            .expect("Unterminated script");

        if !attributes.contains("src=") && !attributes.contains("application/ld+json") {
            scripts.push(content);
        }
        rest = after_end;
    }

    scripts
}

/// Returns a script's hash in Content Security Policy format.
///
/// Browsers convert line endings to LF before hashing, so this must as well.
fn csp_hash(script: &str) -> String {
    let normalised = script.replace("\r\n", "\n").replace('\r', "\n");
    let hash = STANDARD.encode(Sha256::digest(normalised));
    format!("'sha256-{hash}'")
}

/// Returns the contents of the `_headers` file.
///
/// Matching rules are combined and repeated headers joined, so `Cache-Control` rules must not
/// overlap. Anything without a rule is revalidated, as that's Cloudflare's default.
///
/// `'wasm-unsafe-eval'` allows the wasm to compile. In `script-src`, `'unsafe-inline'` is only a
/// fallback for browsers too old for hashes, as browsers that support them ignore it. In
/// `style-src`, it covers the page's style tag and the one `rfd` adds.
///
/// Trusted Types blocks HTML strings, except through the `default` policy in `index.html`, the
/// only policy allowed.
fn headers(script_hashes: &str) -> String {
    format!(
        "\
/*
  Content-Security-Policy: default-src 'none'; script-src 'self' 'wasm-unsafe-eval' {script_hashes} 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self'; connect-src 'self'; manifest-src 'self'; base-uri 'self'; form-action 'none'; frame-ancestors 'none'; require-trusted-types-for 'script'; trusted-types default
  Strict-Transport-Security: max-age=31536000; includeSubDomains
  Cross-Origin-Opener-Policy: same-origin
  Cross-Origin-Embedder-Policy: require-corp
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin

/fe_character_creator-*
  Cache-Control: public, max-age=31536000, immutable

/art/*.png
  Cache-Control: public, max-age=3600

/art/*.gz
  Cache-Control: public, max-age=31536000, immutable
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_scripts_skips_external_and_data_scripts() {
        let html = r#"
            <script type="application/ld+json">{}</script>
            <script type="module">first</script>
            <script src="404-script.js"></script>
            <script>second</script>
        "#;

        assert_eq!(
            inline_scripts(html),
            ["first", "second"],
            "Should only find scripts the browser runs inline"
        );
    }

    #[test]
    fn csp_hash_matches_the_csp_specification() {
        assert_eq!(
            csp_hash("alert('Hello, world.');"),
            "'sha256-qznLcsROx4GACP2dm0UCKCzCG+HiZ1guq6ZZDob/Tng='",
            "Should match the hash given in the CSP specification"
        );
    }

    #[test]
    fn csp_hash_ignores_line_ending_style() {
        assert_eq!(
            csp_hash("a\r\nb\rc"),
            csp_hash("a\nb\nc"),
            "Should hash line endings as the browser does"
        );
    }
}
