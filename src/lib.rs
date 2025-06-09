//! A comprehensive library for parsing, managing, and deduplicating academic citations.
//!
//! `biblib` provides robust functionality for working with academic citations in various formats.
//! It focuses on accurate parsing, format conversion, and intelligent deduplication of citations.
//!
//! # Features
//!
//! The library has several optional features that can be enabled in your Cargo.toml:
//!
//! - `csv` - Enable CSV format support (enabled by default)
//! - `pubmed` - Enable PubMed/MEDLINE format support (enabled by default)  
//! - `xml` - Enable EndNote XML support (enabled by default)
//! - `ris` - Enable RIS format support (enabled by default)
//! - `dedupe` - Enable citation deduplication (enabled by default)
//!
//! To use only specific features, disable default features and enable just what you need:
//!
//! ```toml
//! [dependencies]
//! biblib = { version = "0.2.0", default-features = false, features = ["csv", "ris"] }
//! ```
//!
//! # Key Characteristics
//!
//! - **Multiple Format Support**: Parse citations from:
//!   - RIS (Research Information Systems)
//!   - PubMed/MEDLINE
//!   - EndNote XML
//!   - CSV with configurable mappings
//!
//! - **Source Tracking**: Each parser can track the source of citations
//!   - `with_source()` method available on all parsers
//!   - Source information preserved in Citation objects
//!   - Useful for tracking citation origins
//!
//! - **Rich Metadata Support**:
//!   - Authors with affiliations
//!   - Journal details (name, abbreviation, ISSN)
//!   - DOIs and other identifiers
//!   - Complete citation metadata
//!
//! # Basic Usage
//!
//! ```rust
//! use biblib::{CitationParser, RisParser};
//!
//! // Parse RIS format with source tracking
//! let input = r#"TY  - JOUR
//! TI  - Example Article
//! AU  - Smith, John
//! ER  -"#;
//!
//! let parser = RisParser::new().with_source("Pubmed");
//! let citations = parser.parse(input).unwrap();
//! println!("Title: {}", citations[0].title);
//! println!("Source: {}", citations[0].source.clone().unwrap());
//! ```
//! # Citation Formats
//!
//! Each format has a dedicated parser with format-specific features:
//!
//! ```rust
//! use biblib::{RisParser, PubMedParser, EndNoteXmlParser, csv::CsvParser};
//!
//! // RIS format
//! let ris = RisParser::new();
//!
//! // PubMed format
//! let pubmed = PubMedParser::new().with_source("Pubmed");
//!
//! // EndNote XML
//! let endnote = EndNoteXmlParser::new().with_source("Google Scholar");
//!
//! // CSV format
//! let csv = CsvParser::new().with_source("Cochrane");
//! ```
//!
//! # Citation Deduplication
//!
//! ```rust
//! use biblib::{Citation, CitationParser, RisParser};
//!
//! let ris_input = r#"TY  - JOUR
//! TI  - Example Citation 1
//! AU  - Smith, John
//! ER  -
//!
//! TY  - JOUR
//! TI  - Example Citation 2
//! AU  - Smith, John
//! ER  -"#;
//!
//! let parser = RisParser::new();
//! let mut citations = parser.parse(ris_input).unwrap();
//!
//! // Configure deduplication
//! use biblib::dedupe::{Deduplicator, DeduplicatorConfig};
//!
//! // Configure deduplication
//! let config = DeduplicatorConfig {
//!     group_by_year: true,
//!     run_in_parallel: true,
//!     source_preferences: vec!["PubMed".to_string(), "Cochrane".to_string()],
//! };
//!
//! let deduplicator = Deduplicator::new().with_config(config);
//! let duplicate_groups = deduplicator.find_duplicates(&citations).unwrap();
//!
//! for group in duplicate_groups {
//!     println!("Original: {}", group.unique.title);
//!     for duplicate in group.duplicates {
//!         println!("  Duplicate: {}", duplicate.title);
//!     }
//! }
//! ```
//!
//! # Error Handling
//!
//! The library uses a custom [`Result`] type that wraps [`CitationError`] for consistent
//! error handling across all operations:
//!
//! ```rust
//! use biblib::{CitationParser, RisParser, CitationError};
//!
//! let result = RisParser::new().parse("invalid input");
//! match result {
//!     Ok(citations) => println!("Parsed {} citations", citations.len()),
//!     Err(CitationError::InvalidFormat(msg)) => eprintln!("Parse error: {}", msg),
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```
//!
//! # Performance Considerations
//!
//! - Use year-based grouping for large datasets
//! - Enable parallel processing for better performance
//! - Consider using CSV format for very large datasets
//!
//! # Thread Safety
//!
//! All parser implementations are thread-safe and can be shared between threads.
//! The deduplicator supports parallel processing through the `run_in_parallel` option.

#[cfg(feature = "xml")]
use quick_xml::events::attributes::AttrError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[cfg(feature = "csv")]
extern crate csv as csv_crate;

#[cfg(feature = "csv")]
pub mod csv;
#[cfg(feature = "dedupe")]
pub mod dedupe;
#[cfg(feature = "xml")]
pub mod endnote_xml;
#[cfg(feature = "pubmed")]
pub mod pubmed;
#[cfg(feature = "ris")]
pub mod ris;

// Reexports
#[cfg(feature = "csv")]
pub use csv::CsvParser;
#[cfg(feature = "xml")]
pub use endnote_xml::EndNoteXmlParser;
#[cfg(feature = "pubmed")]
pub use pubmed::PubMedParser;
#[cfg(feature = "ris")]
pub use ris::RisParser;

mod utils;

/// A specialized Result type for citation operations.
pub type Result<T> = std::result::Result<T, CitationError>;

/// Represents errors that can occur during citation parsing.
#[derive(Error, Debug)]
pub enum CitationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value: {field} - {message}")]
    InvalidFieldValue { field: String, message: String },

    #[error("Malformed input: {message} at line {line}")]
    MalformedInput { message: String, line: usize },

    #[error("Error: {0}")]
    Other(String),
}

// Add From implementations for common error types
#[cfg(feature = "csv")]
impl From<csv_crate::Error> for CitationError {
    fn from(err: csv_crate::Error) -> Self {
        CitationError::InvalidFormat(err.to_string())
    }
}
#[cfg(feature = "xml")]
impl From<quick_xml::Error> for CitationError {
    fn from(err: quick_xml::Error) -> Self {
        CitationError::InvalidFormat(err.to_string())
    }
}

#[cfg(feature = "xml")]
impl From<AttrError> for CitationError {
    fn from(err: AttrError) -> Self {
        CitationError::InvalidFormat(err.to_string())
    }
}

/// Represents a publication date with optional day, month, and year components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Date {
    /// Publication year
    pub year: Option<i32>,
    /// Publication month (1-12)
    pub month: Option<u8>,
    /// Publication day (1-31)
    pub day: Option<u8>,
}

impl Default for Date {
    fn default() -> Self {
        Self {
            year: None,
            month: None,
            day: None,
        }
    }
}

/// Represents an author of a citation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Author {
    /// The author's family name (surname)
    pub family_name: String,
    /// The author's given name (first name)
    pub given_name: String,
    /// Optional affiliation
    pub affiliation: Option<String>,
}

/// Represents a single citation with its metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Citation {
    pub id: String,
    /// Type of the citation
    pub citation_type: Vec<String>,
    /// Title of the work
    pub title: String,
    /// List of authors
    pub authors: Vec<Author>,
    /// Journal name
    pub journal: Option<String>,
    /// Journal abbreviation
    pub journal_abbr: Option<String>,
    /// Publication date with day, month, and year
    pub date: Date,
    /// Publication year
    #[deprecated(since = "0.3.0", note = "Use `date.year` instead")]
    pub year: Option<i32>,
    /// Volume number
    pub volume: Option<String>,
    /// Issue number
    pub issue: Option<String>,
    /// Page range
    pub pages: Option<String>,
    /// ISSN of the journal
    pub issn: Vec<String>,
    /// Digital Object Identifier
    pub doi: Option<String>,
    /// PubMed ID
    pub pmid: Option<String>,
    /// PMC ID
    pub pmc_id: Option<String>,
    /// Abstract text
    pub abstract_text: Option<String>,
    /// Keywords
    pub keywords: Vec<String>,
    /// URLs
    pub urls: Vec<String>,
    /// Language
    pub language: Option<String>,
    /// MeSH Terms
    pub mesh_terms: Vec<String>,
    /// Publisher
    pub publisher: Option<String>,
    /// Additional fields not covered by standard fields
    pub extra_fields: HashMap<String, Vec<String>>,
    /// Source of the citation (e.g. pubmed, ris, etc.)
    pub source: Option<String>,
}

/// Represents a group of duplicate citations with one unique citation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    /// The unique (original) citation
    pub unique: Citation,
    /// The duplicate citations
    pub duplicates: Vec<Citation>,
}

/// Trait for implementing citation parsers.
pub trait CitationParser {
    /// Parse a string containing one or more citations.
    ///
    /// # Arguments
    ///
    /// * `input` - The string containing citation data
    ///
    /// # Returns
    ///
    /// A Result containing a vector of parsed Citations or a CitationError
    ///
    /// # Errors
    ///
    /// Returns `CitationError` if the input is malformed
    fn parse(&self, input: &str) -> Result<Vec<Citation>>;
}

/// Format detection and automatic parsing of citation files
///
/// # Arguments
///
/// * `content` - The content of the file to parse
/// * `source` - Source of citations
///
/// # Returns
///
/// A Result containing a vector of parsed Citations and the detected format,
/// or a CitationError if parsing fails
///
/// # Examples
///
/// ```
/// use biblib::detect_and_parse;
///
/// let content = r#"TY  - JOUR
/// TI  - Example Title
/// ER  -"#;
///
/// let (citations, format) = detect_and_parse(content, "Cochrane").unwrap();
/// assert_eq!(format, "RIS");
/// assert_eq!(citations[0].title, "Example Title");
/// assert_eq!(citations[0].source.as_deref(), Some("Cochrane"));
/// ```
pub fn detect_and_parse(content: &str, source: &str) -> Result<(Vec<Citation>, &'static str)> {
    let trimmed = content.trim();

    // Empty content check
    if trimmed.is_empty() {
        return Err(CitationError::InvalidFormat("Empty content".into()));
    }

    // Try to detect format based on content patterns
    if trimmed.starts_with("<?xml") || trimmed.starts_with("<xml>") {
        // EndNote XML format
        #[cfg(feature = "xml")]
        {
            let parser = EndNoteXmlParser::new().with_source(source);
            return parser
                .parse(content)
                .map(|citations| (citations, "EndNote XML"));
        }
        #[cfg(not(feature = "xml"))]
        return Err(CitationError::Other(
            "EndNote XML support not enabled".into(),
        ));
    }

    // Check for RIS format (starts with TY or has TY  - pattern)
    if trimmed.starts_with("TY  -") || trimmed.contains("\nTY  -") {
        #[cfg(feature = "ris")]
        {
            let parser = RisParser::new().with_source(source);
            return parser.parse(content).map(|citations| (citations, "RIS"));
        }
        #[cfg(not(feature = "ris"))]
        return Err(CitationError::Other("RIS support not enabled".into()));
    }

    // Check for PubMed format (starts with PMID- or has PMID- pattern)
    if trimmed.starts_with("PMID-") || trimmed.contains("\nPMID-") {
        #[cfg(feature = "pubmed")]
        {
            let parser = PubMedParser::new().with_source(source);
            return parser.parse(content).map(|citations| (citations, "PubMed"));
        }
        #[cfg(not(feature = "pubmed"))]
        return Err(CitationError::Other("PubMed support not enabled".into()));
    }

    Err(CitationError::InvalidFormat(
        "Unable to detect citation format".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_error_display() {
        let error = CitationError::InvalidFormat("Invalid line".to_string());
        assert_eq!(error.to_string(), "Parse error: Invalid line");
    }

    #[test]
    fn test_author_equality() {
        let author1 = Author {
            family_name: "Smith".to_string(),
            given_name: "John".to_string(),
            affiliation: None,
        };
        let author2 = Author {
            family_name: "Smith".to_string(),
            given_name: "John".to_string(),
            affiliation: None,
        };
        assert_eq!(author1, author2);
    }

    #[test]
    fn test_detect_and_parse_ris() {
        let content = r#"TY  - JOUR
TI  - Test Title
AU  - Smith, John
ER  -"#;

        let (citations, format) = detect_and_parse(content, "Google Scholar").unwrap();
        assert_eq!(format, "RIS");
        assert_eq!(citations[0].title, "Test Title");
        assert_eq!(citations[0].source.as_deref(), Some("Google Scholar"));
    }

    #[test]
    fn test_detect_and_parse_pubmed() {
        let content = r#"PMID- 12345678
TI  - Test Title
FAU - Smith, John"#;

        let (citations, format) = detect_and_parse(content, "Pubmed").unwrap();
        assert_eq!(format, "PubMed");
        assert_eq!(citations[0].title, "Test Title");
        assert_eq!(citations[0].source.as_deref(), Some("Pubmed"));
    }

    #[test]
    fn test_detect_and_parse_endnote() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<xml><records><record>
<titles><title>Test Title</title></titles>
</record></records></xml>"#;

        let (citations, format) = detect_and_parse(content, "Embase").unwrap();
        assert_eq!(format, "EndNote XML");
        assert_eq!(citations[0].title, "Test Title");
        assert_eq!(citations[0].source.as_deref(), Some("Embase"));
    }

    #[test]
    fn test_detect_and_parse_empty() {
        let result = detect_and_parse("", "Any Source");
        assert!(matches!(result, Err(CitationError::InvalidFormat(_))));
    }

    #[test]
    fn test_detect_and_parse_unknown() {
        let content = "Some random content\nthat doesn't match\nany known format";
        let result = detect_and_parse(content, "Unknown");
        assert!(matches!(result, Err(CitationError::InvalidFormat(_))));
    }
}
