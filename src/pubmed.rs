//! PubMed format parser implementation.
//!
//! The PubMed format is used by the National Library of Medicine for citations.
//! This module provides functionality to parse PubMed formatted citations into structured data.
//!
//! # Example
//!
//! ```
//! use biblib::{CitationParser, PubMedParser};
//!
//! let input = r#"PMID- 12345678
//! TI  - Example Title
//! FAU - Smith, John
//!
//! "#;
//!
//! let parser = PubMedParser::new();
//! let citations = parser.parse(input).unwrap();
//! assert_eq!(citations[0].title, "Example Title");
//! ```

use crate::{Author, Citation, CitationParser, Result, CitationError};
use crate::utils::{format_page_numbers, parse_author_name, format_doi};
use nanoid::nanoid;

/// Parser for PubMed format citations.
///
/// PubMed format is commonly used by PubMed and the National Library of Medicine
/// for bibliographic citations.
#[derive(Debug, Clone)]
pub struct PubMedParser;

impl Default for PubMedParser {
    fn default() -> Self {
        Self
    }
}

#[derive(Debug, PartialEq)]
enum PubMedLine<'a> {
    Field { tag: &'a str, content: &'a str },
    Continuation(&'a str),
}

impl PubMedParser {
    /// Creates a new PubMed parser instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use biblib::PubMedParser;
    /// let parser = PubMedParser::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    
    /// Parses an author string in the format "LastName, FirstName".
    ///
    /// ## Arguments
    ///
    /// * `author_str` - The author string to parse
    fn parse_author(author_str: &str) -> Author {
        let (family, given) = parse_author_name(author_str);
        Author {
            family_name: family,
            given_name: given,
            affiliation: None
        }
    }

    /// Handles continued lines in PubMed format.
    ///
    /// # Arguments
    ///
    /// * `citation` - The current citation being built
    /// * `field` - The current field being processed
    /// * `content` - The content to append
    fn handle_continuation(
        citation: &mut Citation,
        field: &str,
        content: &str,
    ) {
        match field {
            "FAU" => citation.authors.push(Self::parse_author(content)),
            "AB" => {
                citation.abstract_text.get_or_insert_with(String::new)
                    .push_str(content);
            }
            "AD" => {
                if let Some(last_author) = citation.authors.last_mut() {
                    if let Some(ref mut aff) = last_author.affiliation {
                        aff.push(' ');
                        aff.push_str(content.trim());
                    } else {
                        last_author.affiliation = Some(content.trim().to_string());
                    }
                }
            }
            _ => {
                if let Some(values) = citation.extra_fields.get_mut(field) {
                    if let Some(last_value) = values.last_mut() {
                        last_value.push(' ');
                        last_value.push_str(content);
                    }
                }
            }
        }
    }

    fn validate_line(line: &str, line_num: usize) -> Result<PubMedLine> {
        let line = line.trim_end();
        
        // Check for continuation line (starts with 6 spaces)
        if let Some(content) = line.strip_prefix("      ") {
            return Ok(PubMedLine::Continuation(content.trim_start()));
        }

        // Parse field line
        if let Some((field, content)) = line.split_once("- ") {
            let field = field.trim();
            if field.chars().all(|c| c.is_ascii_uppercase()) {
                Ok(PubMedLine::Field {
                    tag: field,
                    content: content.trim(),
                })
            } else {
                Err(CitationError::MalformedInput {
                    message: format!("Invalid field tag format: '{}'", field),
                    line: line_num,
                })
            }
        } else {
            Err(CitationError::MalformedInput {
                message: format!("Invalid line format: '{}'", line),
                line: line_num,
            })
        }
    }
}

impl CitationParser for PubMedParser {
    /// Parses a string containing one or more citations in PubMed format.
    ///
    /// # Arguments
    ///
    /// * `input` - The PubMed formatted string to parse
    ///
    /// # Returns
    ///
    /// A Result containing a vector of parsed Citations or a CitationError
    ///
    /// # Errors
    ///
    /// Returns `CitationError` if the input is malformed
    fn parse(&self, input: &str) -> Result<Vec<Citation>> {
        if input.trim().is_empty() {
            return Err(CitationError::InvalidFormat("Empty input".into()));
        }

        let mut citations = Vec::new();
        let mut current_citation = Citation::default();
        current_citation.id = nanoid!();
        let mut current_field = String::new();
        let mut temp_au_authors: Vec<Author> = Vec::new();

        for (line_num, line) in input.lines().enumerate() {
            let line = line.trim_end();
            if line.is_empty() {
                if !current_citation.title.is_empty() {
                    // If we have AU authors but no FAU authors, use the AU authors
                    if current_citation.authors.is_empty() && !temp_au_authors.is_empty() {
                        current_citation.authors = temp_au_authors;
                    }
                    citations.push(current_citation);
                    current_citation = Citation::default();
                    current_citation.id = nanoid!();
                    temp_au_authors = Vec::new();
                }
                continue;
            }

            match Self::validate_line(line, line_num + 1)? {
                PubMedLine::Continuation(content) => {
                    Self::handle_continuation(&mut current_citation, &current_field, content);
                }
                PubMedLine::Field { tag, content } => {
                    current_field = tag.to_string();
                    match tag {
                        "PMID" => current_citation.pmid = Some(content.to_string()),
                        "PMC" => current_citation.pmc_id = Some(content.to_string()),
                        "TI" => current_citation.title = content.to_string(),
                        "JT" => current_citation.journal = Some(content.to_string()),
                        "TA" => current_citation.journal_abbr = Some(content.to_string()),
                        "DP" => if let Ok(year) = content.split_whitespace().next()
                            .unwrap_or("0").parse::<i32>() {
                            current_citation.year = Some(year);
                        },
                        "VI" => current_citation.volume = Some(content.to_string()),
                        "IP" => current_citation.issue = Some(content.to_string()),
                        "PG" => current_citation.pages = Some(format_page_numbers(content)),
                        "LID" => {
                            if content.ends_with("[doi]") || content.contains("doi.org") {
                                current_citation.doi = format_doi(content);
                            }
                        }
                        "PT" => current_citation.citation_type.push(content.to_string()),
                        "FAU" => current_citation.authors.push(Self::parse_author(content)),
                        "AU" => temp_au_authors.push(Self::parse_author(content)),
                        "AB" => current_citation.abstract_text = Some(content.to_string()),
                        "OT" => current_citation.keywords.push(content.to_string()),
                        "MH" => current_citation.mesh_terms.push(content.to_string()),
                        "AD" => if let Some(last_author) = current_citation.authors.last_mut() {
                            last_author.affiliation = Some(content.to_string());
                        },
                        "LA" => current_citation.language = Some(content.to_string()),
                        "IS" => current_citation.issn.push(content.to_string()),
                        _ => {
                            current_citation.extra_fields
                                .entry(tag.to_string())
                                .or_default()
                                .push(content.to_string());
                        }
                    }
                }
            }
        }

        if !current_citation.title.is_empty() {
            if current_citation.authors.is_empty() && !temp_au_authors.is_empty() {
                current_citation.authors = temp_au_authors;
            }
            citations.push(current_citation);
        }

        if citations.is_empty() {
            return Err(CitationError::InvalidFormat("No valid citations found".into()));
        }

        Ok(citations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_citation() {
        let input = r#"PMID- 12345678
TI- Test Article Title
FAU- Smith, John
JT- Test Journal
DP- 2023 Jan
VI- 10
IP- 2
PG- 100-110
LID- 10.1000/test [doi]
AB- This is a test abstract.
MH- Keyword1
MH- Keyword2

"#;
        let parser = PubMedParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result.len(), 1);
        let citation = &result[0];
        assert_eq!(citation.pmid.as_deref(), Some("12345678"));
        assert_eq!(citation.title, "Test Article Title");
        assert_eq!(citation.authors.len(), 1);
        assert_eq!(citation.authors[0].family_name, "Smith");
        assert_eq!(citation.year, Some(2023));
    }

    #[test]
    fn test_parse_citation_with_affiliation() {
        let input = r#"PMID- 12345678
TI  - Test Article Title
FAU - Smith, John
AD  - Department of Science, Test University
      New York, NY 10021, USA
JT  - Test Journal

"#;
        let parser = PubMedParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result[0].authors[0].affiliation.as_deref(), 
            Some("Department of Science, Test University New York, NY 10021, USA"));
    }

    #[test]
    fn test_journal_names() {
        let input = r#"PMID- 12345678
TI  - Test Article
JT  - Journal of Testing
TA  - J Test

"#;
        let parser = PubMedParser::new();
        let result = parser.parse(input).unwrap();
        
        assert_eq!(result[0].journal.as_deref(), Some("Journal of Testing"));
        assert_eq!(result[0].journal_abbr.as_deref(), Some("J Test"));
    }

    #[test]
    fn test_journal_fallback() {
        let input = r#"PMID- 12345678
TI  - Test Article
TA  - J Test

"#;
        let parser = PubMedParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result[0].journal.as_deref(), None);
        assert_eq!(result[0].journal_abbr.as_deref(), Some("J Test"));
    }

    // Add test for ISSN parsing
    #[test]
    fn test_parse_citation_with_issn() {
        let input = r#"PMID- 12345678
TI  - Test Article Title
IS  - 1234-5678
IS  - 8765-4321

"#;
        let parser = PubMedParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result[0].issn, vec!["1234-5678", "8765-4321"]);
    }

    #[test]
    fn test_parse_citation_with_au_tag() {
        let input = r#"PMID- 12345678
TI  - Test Article Title
AU  - Smith J
AU  - Jones B

"#;
        let parser = PubMedParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result[0].authors.len(), 2);
        assert_eq!(result[0].authors[0].family_name, "Smith");
        assert_eq!(result[0].authors[0].given_name, "J");
        assert_eq!(result[0].authors[1].family_name, "Jones");
        assert_eq!(result[0].authors[1].given_name, "B");
    }

    #[test]
    fn test_fau_precedence_over_au() {
        let input = r#"PMID- 12345678
TI  - Test Article Title
FAU - Li, Yun
AU  - Li Y
FAU - Zhang, Huajun
AU  - Zhang H

"#;
        let parser = PubMedParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result[0].authors.len(), 2);
        assert_eq!(result[0].authors[0].family_name, "Li");
        assert_eq!(result[0].authors[0].given_name, "Yun");
        assert_eq!(result[0].authors[1].family_name, "Zhang");
        assert_eq!(result[0].authors[1].given_name, "Huajun");
    }

    #[test]
    fn test_validate_line() {
        assert!(matches!(
            PubMedParser::validate_line("PMID- 12345678", 1).unwrap(),
            PubMedLine::Field { tag: "PMID", content: "12345678" }
        ));

        assert!(matches!(
            PubMedParser::validate_line("      Continuation text", 1).unwrap(),
            PubMedLine::Continuation("Continuation text")
        ));

        assert!(PubMedParser::validate_line("Invalid- line", 1).is_err());
    }
}
