//! RIS format parsing implementation.
//!
//! This module handles the low-level parsing of RIS formatted text.

use crate::ris::structure::RawRisData;
use crate::ris::tags::RisTag;
use crate::utils::parse_author_name;
use crate::{
    Author, CitationFormat,
    error::{ParseError, ValueError},
};

/// Parse the content of a RIS formatted file, returning structured data.
pub(crate) fn ris_parse<S: AsRef<str>>(ris_text: S) -> Result<Vec<RawRisData>, ParseError> {
    let text = ris_text.as_ref();

    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut citations = Vec::new();
    let mut current_citation = RawRisData::new();
    let mut line_number = 0;

    for line in text.lines() {
        line_number += 1;
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Skip metadata lines
        if is_metadata_line(line) {
            continue;
        }

        match parse_ris_line(line, line_number) {
            Ok((tag, content)) => {
                match tag {
                    RisTag::Type => {
                        // Start of new citation
                        if current_citation.has_content() {
                            citations.push(current_citation);
                            current_citation = RawRisData::new();
                        }
                        current_citation.add_data(tag, content);
                    }
                    RisTag::EndOfReference => {
                        // End of current citation
                        if current_citation.has_content() {
                            citations.push(current_citation);
                            current_citation = RawRisData::new();
                        }
                    }
                    tag if tag.is_author_tag() => {
                        let author = parse_author(&content);
                        current_citation.add_author(author);
                    }
                    _ => {
                        current_citation.add_data(tag, content);
                    }
                }
            }
            Err(_) => {
                // Add invalid lines to ignored lines with context
                current_citation.add_ignored_line(line_number, line.to_string());
            }
        }
    }

    // Add the last citation if it has content
    if current_citation.has_content() {
        citations.push(current_citation);
    }

    if citations.is_empty() {
        return Ok(Vec::new());
    }

    Ok(citations)
}

/// Parse a single RIS line into a tag and content.
fn parse_ris_line(line: &str, line_number: usize) -> Result<(RisTag, String), ParseError> {
    // Validate minimum line length
    if line.len() < 2 {
        return Err(ParseError::at_line(
            line_number,
            CitationFormat::Ris,
            ValueError::Syntax(format!(
                "Line too short for RIS format (minimum 2 chars): '{}'",
                line
            )),
        ));
    }

    let tag_str = &line[..2];

    // Validate tag format
    if !tag_str.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ParseError::at_line(
            line_number,
            CitationFormat::Ris,
            ValueError::Syntax(format!("Invalid RIS tag format: '{}'", tag_str)),
        ));
    }

    let tag = RisTag::from_tag(tag_str);

    // Extract content
    let content = extract_ris_content(line, line_number)?;

    Ok((tag, content))
}

/// Extract content from a RIS line, handling various format patterns.
fn extract_ris_content(line: &str, line_number: usize) -> Result<String, ParseError> {
    // Standard format: "TY  - JOUR"
    if line.len() >= 6 && &line[2..6] == "  - " {
        return Ok(line[6..].trim().to_string());
    }

    // Format without space after dash: "ER  -"
    if line.len() >= 5 && &line[2..5] == "  -" {
        return Ok(line[5..].trim().to_string());
    }

    // Format without spaces before dash: "TY- JOUR"
    if line.len() >= 4 && &line[2..4] == "- " {
        return Ok(line[4..].trim().to_string());
    }

    // Minimal format: "TY-JOUR"
    if line.len() >= 3 && &line[2..3] == "-" {
        return Ok(line[3..].trim().to_string());
    }

    // Require proper separator (space or dash) after tag
    if line.len() > 2 {
        let third_char = line.chars().nth(2).unwrap();
        if third_char == ' ' || third_char == '-' {
            return Ok(line[2..].trim().to_string());
        }
    }

    // If we reach here, the line doesn't have a proper separator
    Err(ParseError::at_line(
        line_number,
        CitationFormat::Ris,
        ValueError::Syntax(format!(
            "RIS line missing proper separator (space or dash) after tag: '{}'",
            line
        )),
    ))
}

/// Parse an author string into an Author struct.
fn parse_author(author_str: &str) -> Author {
    let (family, given) = parse_author_name(author_str);
    Author {
        family_name: family,
        given_name: given,
        affiliation: None,
    }
}

/// Check if a line is RIS metadata that should be ignored.
fn is_metadata_line(line: &str) -> bool {
    line.starts_with("Record #")
        || line.starts_with("Provider:")
        || line.starts_with("Content:")
        || line.starts_with("Database:")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("TY  - JOUR", RisTag::Type, "JOUR")]
    #[case("TI  - Test Title", RisTag::Title, "Test Title")]
    #[case("AU  - Smith, John", RisTag::Author, "Smith, John")]
    #[case("ER  -", RisTag::EndOfReference, "")]
    #[case("DO  - 10.1000/test", RisTag::Doi, "10.1000/test")]
    #[case("TY Content", RisTag::Type, "Content")]
    #[case("TY-Content", RisTag::Type, "Content")]
    fn test_parse_ris_line_valid(
        #[case] line: &str,
        #[case] expected_tag: RisTag,
        #[case] expected_content: &str,
    ) {
        let result = parse_ris_line(line, 1).unwrap();
        assert_eq!(result.0, expected_tag);
        assert_eq!(result.1, expected_content);
    }

    #[rstest]
    #[case("")]
    #[case("A")]
    #[case("!!  - Invalid tag")]
    #[case("TYNoSeparator")]
    #[case("TYBAD")]
    fn test_parse_ris_line_invalid(#[case] line: &str) {
        let result = parse_ris_line(line, 1);
        assert!(result.is_err());
    }

    #[rstest]
    #[case("Record #1 of 10", true)]
    #[case("Provider: Some Provider", true)]
    #[case("Content: text/plain", true)]
    #[case("Database: PubMed", true)]
    #[case("TY  - JOUR", false)]
    fn test_is_metadata_line(#[case] line: &str, #[case] expected: bool) {
        assert_eq!(is_metadata_line(line), expected);
    }

    #[test]
    fn test_parse_simple_citation() {
        let input = r#"TY  - JOUR
TI  - Test Article
AU  - Smith, John
ER  -"#;

        let result = ris_parse(input).unwrap();
        assert_eq!(result.len(), 1);

        let raw = &result[0];
        assert_eq!(raw.get_first(&RisTag::Type), Some(&"JOUR".to_string()));
        assert_eq!(
            raw.get_first(&RisTag::Title),
            Some(&"Test Article".to_string())
        );
        assert_eq!(raw.authors.len(), 1);
        assert_eq!(raw.authors[0].family_name, "Smith");
    }

    #[test]
    fn test_parse_multiple_citations() {
        let input = r#"TY  - JOUR
TI  - First Article
AU  - Smith, John
ER  -

TY  - BOOK
TI  - Second Article
AU  - Doe, Jane
ER  -"#;

        let result = ris_parse(input).unwrap();
        assert_eq!(result.len(), 2);

        assert_eq!(
            result[0].get_first(&RisTag::Type),
            Some(&"JOUR".to_string())
        );
        assert_eq!(
            result[1].get_first(&RisTag::Type),
            Some(&"BOOK".to_string())
        );
    }

    #[test]
    fn test_parse_with_metadata() {
        let input = r#"Record #1 of 2
Provider: Test Provider
Database: Test DB

TY  - JOUR
TI  - Test Article
AU  - Smith, John
ER  -"#;

        let result = ris_parse(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get_first(&RisTag::Title),
            Some(&"Test Article".to_string())
        );
    }

    #[test]
    fn test_parse_empty_input() {
        let result = ris_parse("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_no_valid_citations() {
        let input = r#"Record #1 of 0
Provider: Test Provider"#;

        let result = ris_parse(input).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_with_invalid_lines() {
        let input = r#"TY  - JOUR
TI  - Test Article
!! - This is truly invalid
AU  - Smith, John
ER  -"#;

        let result = ris_parse(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].ignored_lines.len(), 1);
        assert!(result[0].ignored_lines[0].1.contains("!!"));
    }

    #[test]
    fn test_parse_author() {
        let author = parse_author("Smith, John");
        assert_eq!(author.family_name, "Smith");
        assert_eq!(author.given_name, "John");
        assert_eq!(author.affiliation, None);
    }
}
