//! CSV format parser implementation.
//!
//! This module provides functionality to parse CSV formatted citations with configurable headers.
//!
//! # Example
//!
//! ```
//! use biblib::{CitationParser, CsvParser};
//!
//! let input = "Title,Author,Year\nExample Paper,Smith J,2023";
//!
//! let parser = CsvParser::new();
//!     
//! let citations = parser.parse(input).unwrap();
//! assert_eq!(citations[0].title, "Example Paper");
//! ```

use csv::{ReaderBuilder, StringRecord};
use nanoid::nanoid;
use std::collections::HashMap;

use crate::utils::{
    format_doi, format_page_numbers, parse_author_name, parse_year_only, split_issns,
};
use crate::{Author, Citation, CitationError, CitationParser, Result};

/// Default header mappings for common CSV column names
const DEFAULT_HEADERS: &[(&str, &[&str])] = &[
    ("id", &["id", "citation_id"]),
    ("title", &["title", "article title", "publication title"]),
    ("authors", &["author", "authors", "creator", "creators"]),
    (
        "journal",
        &["journal", "journal title", "source title", "publication"],
    ),
    ("year", &["year", "publication year", "pub year"]),
    ("volume", &["volume", "vol"]),
    ("issue", &["issue", "number", "no"]),
    ("pages", &["pages", "page numbers", "page range"]),
    ("doi", &["doi", "digital object identifier"]),
    ("abstract", &["abstract", "summary"]),
    ("keywords", &["keywords", "tags"]),
    ("issn", &["issn", "isbn"]),
    ("language", &["language", "lang"]),
    ("publisher", &["publisher"]),
    ("url", &["url", "link", "web link"]),
    ("label", &["label"]),
    ("duplicate_id", &["duplicateid", "duplicate_id"]),
];

/// Configuration for CSV parsing with custom header mappings.
///
/// Allows customization of how CSV columns are mapped to citation fields,
/// along with general CSV parsing options like delimiters and header presence.
///
/// # Default Mappings
///
/// The default configuration includes mappings for common column names:
/// - "title" → ["title", "article title", "publication title"]
/// - "authors" → ["author", "authors", "creator", "creators"]
/// - "year" → ["year", "publication year", "pub year"]
///   etc.
///
/// # Examples
///
/// ```
/// use biblib::csv::CsvConfig;
///
/// let mut config = CsvConfig::new();
/// config.set_header_mapping("title", vec!["Article Name".to_string()]);
/// config.set_delimiter(b';');
/// ```
#[derive(Debug, Clone, Default)]
pub struct CsvConfig {
    /// Custom header mappings for CSV columns
    header_map: HashMap<String, Vec<String>>,
    /// Delimiter to use for parsing the CSV
    delimiter: u8,
    /// Whether the CSV has headers
    has_header: bool,
}

impl CsvConfig {
    /// Creates a new CSV configuration with default settings
    #[must_use]
    pub fn new() -> Self {
        let mut config = Self {
            header_map: HashMap::new(),
            delimiter: b',',
            has_header: true,
        };
        config.set_default_headers();
        config
    }

    /// Sets the default header mappings
    fn set_default_headers(&mut self) {
        for (field, aliases) in DEFAULT_HEADERS {
            self.header_map.insert(
                field.to_string(),
                aliases.iter().map(|s| s.to_string()).collect(),
            );
        }
    }

    /// Sets a custom header mapping
    pub fn set_header_mapping(&mut self, field: &str, aliases: Vec<String>) -> &mut Self {
        self.header_map.insert(field.to_string(), aliases);
        self
    }

    /// Sets the delimiter character
    pub fn set_delimiter(&mut self, delimiter: u8) -> &mut Self {
        self.delimiter = delimiter;
        self
    }

    /// Sets whether the CSV has headers
    pub fn set_has_header(&mut self, has_header: bool) -> &mut Self {
        self.has_header = has_header;
        self
    }

    /// Finds the field name for a given header
    fn get_field_for_header(&self, header: &str) -> Option<String> {
        let header_lower = header.to_lowercase();
        self.header_map
            .iter()
            .find(|(_, aliases)| aliases.iter().any(|a| a.to_lowercase() == header_lower))
            .map(|(field, _)| field.clone())
    }
}

/// Parser for CSV-formatted citation data with configurable mappings.
///
/// Provides flexible parsing of CSV files containing citation data, with support
/// for custom column mappings and different CSV dialects.
///
/// # Features
///
/// - Custom header mappings
/// - Configurable delimiters
/// - Multiple author parsing
/// - Support for duplicate groups
///
/// # Examples
///
/// Basic usage:
/// ```
/// use biblib::{CsvParser, CitationParser};
///
/// let input = "Title,Author,Year\nExample Paper,Smith J,2023";
/// let parser = CsvParser::new();
/// let citations = parser.parse(input).unwrap();
/// ```
///
/// With custom configuration:
/// ```
/// use biblib::csv::{CsvParser, CsvConfig};
///
/// let mut config = CsvConfig::new();
/// config.set_delimiter(b';');
///
/// let parser = CsvParser::new().with_config(config);
/// ```
#[derive(Debug, Clone)]
pub struct CsvParser {
    config: CsvConfig,
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CsvParser {
    /// Creates a new CSV parser with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: CsvConfig::new(),
        }
    }

    /// Creates a new CSV parser with custom configuration
    #[must_use]
    pub fn with_config(mut self, config: CsvConfig) -> Self {
        self.config = config;
        self
    }

    /// Parses a record into a Citation using the current header mapping
    fn parse_record(&self, headers: &[String], record: StringRecord) -> Result<Citation> {
        let mut citation = Citation {
            ..Default::default()
        };
        let mut has_id = false;

        for (i, value) in record.iter().enumerate() {
            if i >= headers.len() {
                break;
            }
            if let Some(field) = self.config.get_field_for_header(&headers[i]) {
                match field.as_str() {
                    "id" => {
                        if !value.is_empty() {
                            citation.id = value.to_string();
                            has_id = true;
                        }
                    }
                    "title" => citation.title = value.to_string(),
                    "authors" => {
                        for author_str in value.split(';') {
                            let (family, given) = parse_author_name(author_str);
                            citation.authors.push(Author {
                                family_name: family,
                                given_name: given,
                                affiliation: None,
                            });
                        }
                    }
                    "journal" => citation.journal = Some(value.to_string()),
                    "year" => {
                        citation.date = parse_year_only(value);
                        // For backward compatibility, also set the deprecated year field
                        #[allow(deprecated)]
                        {
                            citation.year = citation.date.as_ref().map(|d| d.year);
                        }
                    }
                    "volume" => citation.volume = Some(value.to_string()),
                    "issue" => citation.issue = Some(value.to_string()),
                    "pages" => citation.pages = Some(format_page_numbers(value)),
                    "doi" => citation.doi = format_doi(value),
                    "abstract" => citation.abstract_text = Some(value.to_string()),
                    "keywords" => {
                        citation.keywords.extend(
                            value
                                .split(';')
                                .map(str::trim)
                                .filter(|s| !s.is_empty())
                                .map(String::from),
                        );
                    }
                    "issn" => {
                        citation.issn.extend(split_issns(value));
                    }
                    "language" => citation.language = Some(value.to_string()),
                    "publisher" => citation.publisher = Some(value.to_string()),
                    "url" => citation.urls.push(value.to_string()),
                    _ => {
                        citation
                            .extra_fields
                            .entry(field)
                            .or_default()
                            .push(value.to_string());
                    }
                }
            }
        }

        if !has_id {
            citation.id = nanoid!();
        }

        Ok(citation)
    }
}

impl CitationParser for CsvParser {
    fn parse(&self, input: &str) -> Result<Vec<Citation>> {
        let mut reader = ReaderBuilder::new()
            .delimiter(self.config.delimiter)
            .has_headers(self.config.has_header)
            .from_reader(input.as_bytes());

        let headers: Vec<String> = if self.config.has_header {
            reader
                .headers()
                .map_err(|e| CitationError::InvalidFormat(e.to_string()))?
                .iter()
                .map(String::from)
                .collect()
        } else {
            // Use column numbers as headers if no headers present
            (0..reader
                .headers()
                .map_err(|e| CitationError::InvalidFormat(e.to_string()))?
                .len())
                .map(|i| format!("Column{}", i + 1))
                .collect()
        };

        let mut citations = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| CitationError::InvalidFormat(e.to_string()))?;
            citations.push(self.parse_record(&headers, record)?);
        }

        Ok(citations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_csv() {
        let input = "\
Title,Author,Year,Journal
Test Paper,Smith J,2023,Test Journal
Another Paper,\"Doe, Jane\",2022,Another Journal";

        let parser = CsvParser::new();
        let citations = parser.parse(input).unwrap();
        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0].title, "Test Paper");
        assert_eq!(citations[0].authors[0].family_name, "Smith");
        assert_eq!(citations[0].date.as_ref().unwrap().year, 2023);
        assert_eq!(citations[0].journal, Some("Test Journal".to_string()));
    }

    #[test]
    fn test_custom_headers() {
        let input = "\
Article Name,Writers,Published,Source
Test Paper,Smith J,2023,Test Journal";

        let mut config = CsvConfig::new();
        config
            .set_header_mapping("title", vec!["Article Name".to_string()])
            .set_header_mapping("authors", vec!["Writers".to_string()])
            .set_header_mapping("year", vec!["Published".to_string()])
            .set_header_mapping("journal", vec!["Source".to_string()]);

        let parser = CsvParser::new().with_config(config);
        let citations = parser.parse(input).unwrap();
        assert_eq!(citations[0].title, "Test Paper");
        assert_eq!(citations[0].authors[0].family_name, "Smith");
        assert_eq!(citations[0].date.as_ref().unwrap().year, 2023);
        assert_eq!(citations[0].journal, Some("Test Journal".to_string()));
    }

    #[test]
    fn test_multiple_authors() {
        let input = "\
Title,Authors,Year
Test Paper,\"Smith, John; Doe, Jane\",2023";

        let parser = CsvParser::new();
        let citations = parser.parse(input).unwrap();

        assert_eq!(citations[0].authors.len(), 2);
        assert_eq!(citations[0].authors[0].family_name, "Smith");
        assert_eq!(citations[0].authors[1].family_name, "Doe");
    }

    #[test]
    fn test_custom_delimiter() {
        let input = "Title;Author;Year\nTest Paper;Smith J;2023";

        let mut config = CsvConfig::new();
        config.set_delimiter(b';');

        let parser = CsvParser::new().with_config(config);
        let citations = parser.parse(input).unwrap();
        assert_eq!(citations[0].title, "Test Paper");
        assert_eq!(citations[0].authors[0].family_name, "Smith");
        assert_eq!(citations[0].date.as_ref().unwrap().year, 2023);
    }
}
