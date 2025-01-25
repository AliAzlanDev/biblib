//! A comprehensive library for parsing, managing, and deduplicating academic citations.
//!
//! `biblib` provides robust functionality for working with academic citations in various formats.
//! It focuses on accurate parsing, format conversion, and intelligent deduplication of citations.
//!
//! # Key Features
//!
//! - **Multiple Format Support**: Parse citations from:
//!   - RIS (Research Information Systems)
//!   - PubMed/MEDLINE
//!   - EndNote XML
//!   - CSV with configurable mappings
//!
//! - **Intelligent Deduplication**:
//!   - DOI-based matching
//!   - Smart title comparison
//!   - Journal name/abbreviation matching
//!   - Configurable matching thresholds
//!   - Parallel processing support
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
//! // Parse RIS format
//! let input = r#"TY  - JOUR
//! TI  - Example Article
//! AU  - Smith, John
//! ER  -"#;
//!
//! let parser = RisParser::new();
//! let citations = parser.parse(input).unwrap();
//! println!("Title: {}", citations[0].title);
//! ```
//!
//! # Citation Deduplication
//!
//! ```rust
//! use biblib::dedupe::{Deduplicator, DeduplicatorConfig};
//!
//! // Configure deduplication
//! let config = DeduplicatorConfig {
//!     group_by_year: true,
//!     run_in_parallel: true,
//! };
//!
//! let deduplicator = Deduplicator::with_config(config);
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
//! # CSV Support with Custom Mappings
//!
//! ```rust
//! use biblib::{CitationParser, csv::CsvParser};
//!
//! let input = "Title,Authors,Year\nExample Paper,Smith J,2023";
//! let parser = CsvParser::new();
//! let citations = parser.parse(input).unwrap();
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
//!     Ok(citations) => println!("Parsed {} citations"),
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

use quick_xml::events::attributes::AttrError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

extern crate csv as csv_crate;

pub mod csv;
pub mod dedupe;
pub mod endnote_xml;
pub mod pubmed;
pub mod ris;
mod utils;

// Reexports
pub use csv::CsvParser;
pub use endnote_xml::EndNoteXmlParser;
pub use pubmed::PubMedParser;
pub use ris::RisParser;

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
impl From<csv_crate::Error> for CitationError {
    fn from(err: csv_crate::Error) -> Self {
        CitationError::InvalidFormat(err.to_string())
    }
}

impl From<quick_xml::Error> for CitationError {
    fn from(err: quick_xml::Error) -> Self {
        CitationError::InvalidFormat(err.to_string())
    }
}

impl From<AttrError> for CitationError {
    fn from(err: AttrError) -> Self {
        CitationError::InvalidFormat(err.to_string())
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
    /// Publication year
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
    /// Label indicating if this is a unique or duplicate citation
    pub label: Option<String>,
    /// ID linking duplicate citations together
    pub duplicate_id: Option<String>,
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
}
