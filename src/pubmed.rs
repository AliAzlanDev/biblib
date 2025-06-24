//! PubMed format parser implementation.
//!
//! Provides functionality to parse PubMed formatted citations.
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
//!     
//! let citations = parser.parse(input).unwrap();
//! assert_eq!(citations[0].title, "Example Title");
//! ```

mod author;
mod parse;
mod structure;
mod tags;

use crate::pubmed::parse::pubmed_parse;
use crate::{Citation, CitationParser, Result};

/// Parser for PubMed format citations.
///
/// PubMed format is commonly used by PubMed and the National Library of Medicine
/// for bibliographic citations.
#[derive(Debug, Clone, Default)]
pub struct PubMedParser {}

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
        let raw_data = pubmed_parse(input);
        let citation = raw_data.try_into()?;
        Ok(vec![citation])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_simple_citation() {
        let input = r#"PMID- 12345678
TI- Test Article Title
FAU- Smith, John
JT- Test Journal
DP- 2023 Jan 23
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
        let date = citation.date.as_ref().unwrap();
        assert_eq!(date.year, 2023);
        assert_eq!(date.month, Some(1));
        assert_eq!(date.day, Some(23));
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
        assert_eq!(
            result[0].authors[0].affiliation.as_deref(),
            Some("Department of Science, Test University New York, NY 10021, USA")
        );
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
    fn test_crlf_endings() {
        let input = "PMID- 123\r\nTI- Windows\r\nFAU- Gates, Bill\r\nFAU- Cutler, Dave";
        let parser = PubMedParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result[0].pmid.as_deref(), Some("123"));
        assert_eq!(result[0].title, "Windows");
        assert_eq!(result[0].authors[0].given_name, "Bill");
        assert_eq!(result[0].authors[0].family_name, "Gates");
        assert_eq!(result[0].authors[1].given_name, "Dave");
        assert_eq!(result[0].authors[1].family_name, "Cutler");
    }

    #[test]
    fn test_continued_line() {
        let input = r#"PMID- 31181385
DP  - 2019 Dec
TI  - Fantastic yeasts and where to find them: the hidden diversity of dimorphic fungal 
      pathogens.
AB  - This is a long abstract that spans
      multiple lines for testing purposes.
FAU - Van Dyke, Marley C Caballero
AU  - Van Dyke MCC
"#;
        let parser = PubMedParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result.len(), 1);
        let citation = &result[0];
        assert_eq!(citation.pmid.as_deref(), Some("31181385"));
        assert_eq!(
            citation.title,
            "Fantastic yeasts and where to find them: the hidden diversity of dimorphic fungal pathogens."
        );
        assert_eq!(
            result[0].abstract_text.as_deref(),
            Some("This is a long abstract that spans multiple lines for testing purposes.")
        );
        assert_eq!(citation.authors.len(), 1);
    }
}
