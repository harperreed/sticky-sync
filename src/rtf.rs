// ABOUTME: RTF text extraction for search indexing
// ABOUTME: Simple regex-based stripper to extract plain text from RTF data

use regex::Regex;

pub fn extract_text(rtf: &str) -> String {
    // Remove RTF control sequences like \rtf1, \ansi, etc.
    let control_re = Regex::new(r"\\[a-z]+[0-9]*\s*").unwrap();
    let cleaned = control_re.replace_all(rtf, " ");

    // Remove braces
    let cleaned = cleaned.replace('{', "").replace('}', "");

    // Collapse multiple spaces
    let space_re = Regex::new(r"\s+").unwrap();
    let cleaned = space_re.replace_all(&cleaned, " ");

    cleaned.trim().to_string()
}

pub fn extract_text_from_bytes(rtf_data: &[u8]) -> String {
    match String::from_utf8(rtf_data.to_vec()) {
        Ok(s) => extract_text(&s),
        Err(_) => String::new(),
    }
}