use sticky_situation::rtf::extract_text;

#[test]
fn test_extract_plain_text() {
    let rtf = r"{\rtf1\ansi\ansicpg1252 Hello World}";
    let text = extract_text(rtf);
    assert!(text.contains("Hello World"));
}

#[test]
fn test_extract_with_formatting() {
    let rtf = r"{\rtf1\ansi{\b Bold} and {\i Italic} text}";
    let text = extract_text(rtf);
    assert!(text.contains("Bold"));
    assert!(text.contains("Italic"));
}

#[test]
fn test_extract_text_from_bytes() {
    use sticky_situation::rtf::extract_text_from_bytes;

    let rtf = b"{\\rtf1\\ansi Test Content}";
    let text = extract_text_from_bytes(rtf);
    assert!(text.contains("Test Content"));
}

#[test]
fn test_display_extracted_text() {
    let rtf = r"{\rtf1\ansi\ansicpg1252 Hello World}";
    let text = extract_text(rtf);
    println!("Extracted from simple RTF: '{}'", text);

    let rtf2 = r"{\rtf1\ansi{\b Bold} and {\i Italic} text}";
    let text2 = extract_text(rtf2);
    println!("Extracted from formatted RTF: '{}'", text2);
}
