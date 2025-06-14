//! RIS format parser implementation with source tracking support.
//!
//! Provides functionality to parse RIS formatted citations with built-in source tracking.
//!
//! # Example
//!
//! ```
//! use biblib::{CitationParser, RisParser};
//!
//! let input = r#"TY  - JOUR
//! TI  - Example Title
//! AU  - Smith, John
//! ER  -"#;
//!
//! let parser = RisParser::new()
//!     .with_source("Google Scholar");
//!     
//! let citations = parser.parse(input).unwrap();
//! assert_eq!(citations[0].title, "Example Title");
//! assert_eq!(citations[0].source.as_deref(), Some("Google Scholar"));
//! ```

use crate::utils::{format_doi, format_page_numbers, parse_author_name, parse_ris_date};
use crate::{Author, Citation, CitationError, CitationParser, Result};
use nanoid::nanoid;

/// Parser for RIS format citations.
///
/// RIS is a standardized format for bibliographic citations that uses two-letter
/// tags at the start of each line to denote different citation fields.
#[derive(Debug, Default, Clone)]
pub struct RisParser {
    source: Option<String>,
}

impl RisParser {
    /// Creates a new RIS parser instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use biblib::RisParser;
    /// let parser = RisParser::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { source: None }
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// Parses an author string in various formats
    fn parse_author(author_str: &str) -> Author {
        let (family, given) = parse_author_name(author_str);
        Author {
            family_name: family,
            given_name: given,
            affiliation: None,
        }
    }

    /// Checks if a line is RIS metadata that should be ignored
    ///
    /// # Arguments
    ///
    /// * `line` - The line to check
    fn is_metadata_line(line: &str) -> bool {
        line.starts_with("Record #")
            || line.starts_with("Provider:")
            || line.starts_with("Content:")
            || line.trim().is_empty()
    }

    /// Validates that a line meets the minimum RIS format requirements.
    ///
    /// # Arguments
    ///
    /// * `line` - The line to validate
    ///
    /// # Returns
    ///
    /// * `Ok((tag, content))` if the line is valid
    /// * `Err(CitationError)` if the line is invalid
    fn validate_line(line: &str) -> Result<(&str, &str)> {
        if line.trim().is_empty() {
            return Err(CitationError::InvalidFormat("Empty line".into()));
        }

        if line.len() < 2 {
            return Err(CitationError::InvalidFormat(format!(
                "Line too short for RIS format (min 2 chars): '{}'",
                line
            )));
        }

        let tag = &line[..2];
        if !tag.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(CitationError::InvalidFormat(format!(
                "Invalid RIS tag format: '{}'",
                tag
            )));
        }

        let content = if line.len() > 6 && &line[2..6] == "  - " {
            line[6..].trim()
        } else {
            line[2..].trim()
        };

        Ok((tag, content))
    }
}

impl CitationParser for RisParser {
    fn parse(&self, input: &str) -> Result<Vec<Citation>> {
        if input.trim().is_empty() {
            return Err(CitationError::InvalidFormat("Empty input".into()));
        }

        let mut citations = Vec::new();
        let mut current_citation = Citation {
            id: nanoid!(),
            source: self.source.clone(),
            ..Default::default()
        };
        current_citation.source = self.source.clone(); // Add source if provided
        let mut start_page = String::new();

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines without error
            if line.is_empty() {
                continue;
            }

            // Skip metadata lines without error
            if Self::is_metadata_line(line) {
                continue;
            }

            match Self::validate_line(line) {
                Ok((tag, content)) => {
                    match tag {
                        "TY" => {
                            if !current_citation.title.is_empty() {
                                citations.push(current_citation);
                                current_citation = Citation::default();
                                current_citation.id = nanoid!();
                            }
                            current_citation.citation_type.push(content.to_string());
                        }
                        "TI" => current_citation.title = content.to_string(),
                        "T1" => {
                            if current_citation.title.is_empty() {
                                current_citation.title = content.to_string()
                            }
                        }
                        "AU" | "A1" | "A2" | "A3" | "A4" => {
                            current_citation.authors.push(Self::parse_author(content))
                        }
                        "JF" | "T2" => current_citation.journal = Some(content.to_string()),
                        "JA" | "J2" => current_citation.journal_abbr = Some(content.to_string()),
                        "JO" => {
                            if current_citation.journal_abbr.is_none() {
                                current_citation.journal_abbr = Some(content.to_string())
                            }
                        }
                        "PY" | "Y1" | "Y2" => {
                            current_citation.date = parse_ris_date(content);
                            // For backward compatibility, also set the deprecated year field
                            #[allow(deprecated)]
                            {
                                current_citation.year =
                                    current_citation.date.as_ref().map(|d| d.year);
                            }
                        }
                        "VL" => current_citation.volume = Some(content.to_string()),
                        "IS" => current_citation.issue = Some(content.to_string()),
                        "SP" => {
                            start_page = content.to_string();
                            // Set pages immediately for single page citations
                            current_citation.pages = Some(format_page_numbers(content));
                        }
                        "EP" => {
                            let page_str = if !start_page.is_empty() {
                                format!("{}-{}", start_page, content)
                            } else {
                                content.to_string()
                            };
                            current_citation.pages = Some(format_page_numbers(&page_str));
                        }
                        "DO" => current_citation.doi = format_doi(content.trim()),
                        "ID" => current_citation.pmid = Some(content.to_string()),
                        "AB" => current_citation.abstract_text = Some(content.to_string()),
                        "N2" => {
                            if current_citation.abstract_text.is_none() {
                                current_citation.abstract_text = Some(content.to_string())
                            }
                        }
                        "KW" => current_citation.keywords.push(content.to_string()),
                        "SN" => current_citation.issn.push(content.to_string()),
                        "L1" | "L2" | "L3" | "L4" | "UR" | "LK" => {
                            if current_citation.doi.is_none() && content.contains("doi.org") {
                                current_citation.doi = format_doi(content);
                            }
                            current_citation.urls.push(content.to_string());
                        }
                        "LA" => current_citation.language = Some(content.to_string()),
                        "PB" => current_citation.publisher = Some(content.to_string()),
                        "ER" => {
                            if !current_citation.title.is_empty() {
                                citations.push(current_citation);
                                current_citation = Citation::default();
                                current_citation.id = nanoid!();
                            }
                        }
                        "C2" => {
                            if content.contains("PMC") {
                                current_citation.pmc_id = Some(content.to_string());
                            }
                        }
                        _ => {
                            current_citation
                                .extra_fields
                                .entry(tag.to_string())
                                .or_default()
                                .push(content.to_string());
                        }
                    }
                }
                Err(_) => continue, // Skip invalid lines without failing
            }
        }

        if !current_citation.title.is_empty() {
            citations.push(current_citation);
        }

        if citations.is_empty() {
            return Err(CitationError::InvalidFormat(
                "No valid citations found".into(),
            ));
        }

        Ok(citations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_ris() {
        let input = r#"TY  - JOUR
TI  - Test Article Title
AU  - Smith, John
JO  - Test Journal
PY  - 2023/12/25/Christmas edition
VL  - 10
IS  - 2
SP  - 100
EP  - 110
DO  - 10.1000/test
AB  - This is a test abstract.
KW  - Keyword1
KW  - Keyword2
ER  -

"#;
        let parser = RisParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result.len(), 1);
        let citation = &result[0];
        assert_eq!(citation.citation_type[0], "JOUR");
        assert_eq!(citation.title, "Test Article Title");
        assert_eq!(citation.authors.len(), 1);
        assert_eq!(citation.authors[0].family_name, "Smith");
        let date = citation.date.as_ref().unwrap();
        assert_eq!(date.year, 2023);
        assert_eq!(date.month, Some(12));
        assert_eq!(date.day, Some(25));
        assert_eq!(citation.pages, Some("100-110".to_string()));
        assert_eq!(citation.keywords.len(), 2);
    }

    #[test]
    fn test_parse_with_metadata() {
        let input = r#"Record #1 of 2
Provider: Some Provider
Content: text/plain; charset="UTF-8"

TY  - JOUR
TI  - Test Article
AU  - Smith, John
ER  -

Record #2 of 2
Provider: Some Provider
Content: text/plain; charset="UTF-8"

TY  - BOOK
TI  - Another Test
AU  - Doe, Jane
ER  -"#;

        let parser = RisParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "Test Article");
        assert_eq!(result[1].title, "Another Test");
    }

    #[test]
    fn test_parse_gs() {
        let input = r#"TY  - JOUR
T1  - Albendazole therapy in children with focal seizures and single small enhancing computerized tomographic lesions: a randomized, placebo-controlled, double blind trial
A1  - Baranwal, Arun K
A1  - Singhi, Pratibha D
A1  - Khandelwal, N
A1  - Singhi, Sunit C
JO  - The Pediatric infectious disease journal
VL  - 17
IS  - 8
SP  - 696
EP  - 700
SN  - 0891-3668
Y1  - 1998///
PB  - LWW
ER  - 


TY  - JOUR
T1  - High-dose praziquantel with cimetidine for refractory neurocysticercosis: a case report with clinical and MRI follow-up.
A1  - Yee, Thomas
A1  - Barakos, Jerome A
A1  - Knight, Robert T
JO  - Western journal of medicine
VL  - 170
IS  - 2
SP  - 112
Y1  - 1999
PB  - BMJ Publishing Group
ER  - 

"#;
        let parser = RisParser::new();
        let citations = parser.parse(&input).unwrap();
        assert_eq!(citations.len(), 2, "Expected 2 citations in test.ris");
        assert_eq!(citations[0].date.as_ref().unwrap().year, 1998);
    }
}
