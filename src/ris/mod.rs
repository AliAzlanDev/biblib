//! RIS format parser implementation.
//!
//! Provides functionality to parse RIS formatted citations with improved structure
//! and error handling.
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
//! let parser = RisParser::new();
//!     
//! let citations = parser.parse(input).unwrap();
//! assert_eq!(citations[0].title, "Example Title");
//! ```

mod parse;
mod structure;
mod tags;

use crate::{Citation, CitationParser, Result};
use parse::ris_parse;

/// Parser for RIS format citations.
///
/// RIS is a standardized format for bibliographic citations that uses two-letter
/// tags at the start of each line to denote different citation fields.
#[derive(Debug, Clone, Default)]
pub struct RisParser;

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
        Self::default()
    }
}

impl CitationParser for RisParser {
    /// Parses a string containing one or more citations in RIS format.
    ///
    /// # Arguments
    ///
    /// * `input` - The RIS formatted string to parse
    ///
    /// # Returns
    ///
    /// A Result containing a vector of parsed Citations or a CitationError
    ///
    /// # Errors
    ///
    /// Returns `CitationError` if the input is malformed or contains no valid citations
    fn parse(&self, input: &str) -> Result<Vec<Citation>> {
        let raw_citations = ris_parse(input)?;

        let mut citations = Vec::with_capacity(raw_citations.len());
        for raw in raw_citations {
            let citation = raw.try_into()?;
            citations.push(citation);
        }

        Ok(citations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
    fn test_parse_gs_format() {
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
        assert_eq!(
            citations.len(),
            2,
            "Expected 2 citations in Google Scholar format"
        );
        assert_eq!(citations[0].date.as_ref().unwrap().year, 1998);
        assert_eq!(citations[1].date.as_ref().unwrap().year, 1999);
    }

    #[test]
    fn test_parse_journal_priority() {
        let input = r#"TY  - JOUR
TI  - Test Article
JO  - Alternative Journal
JF  - Main Journal
T2  - Secondary Title
ER  -"#;

        let parser = RisParser::new();
        let result = parser.parse(input).unwrap();

        // JF (JournalFull) should have priority over JO (JournalFullAlternative)
        assert_eq!(result[0].journal, Some("Main Journal".to_string()));
    }

    #[test]
    fn test_parse_url_with_doi_extraction() {
        let input = r#"TY  - JOUR
TI  - Test Article
UR  - https://doi.org/10.1000/test
L1  - https://example.com/pdf
ER  -"#;

        let parser = RisParser::new();
        let result = parser.parse(input).unwrap();

        assert_eq!(result[0].urls.len(), 2);
        assert!(
            result[0]
                .urls
                .contains(&"https://doi.org/10.1000/test".to_string())
        );
        assert!(
            result[0]
                .urls
                .contains(&"https://example.com/pdf".to_string())
        );
    }

    #[test]
    fn test_parse_empty_input() {
        let parser = RisParser::new();
        let result = parser.parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_no_valid_citations() {
        let input = r#"Record #1 of 0
Provider: Test Provider
Content: metadata only"#;

        let parser = RisParser::new();
        let result = parser.parse(input);
        assert!(result.is_err());
    }
}
