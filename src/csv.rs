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
//! let parser = CsvParser::new();
//! let citations = parser.parse(input).unwrap();
//! assert_eq!(citations[0].title, "Example Paper");
//! ```

use csv::{ReaderBuilder, StringRecord};
use nanoid::nanoid;
use std::collections::HashMap;

use crate::utils::{format_doi, format_page_numbers, parse_author_name, split_issns};
use crate::{Author, Citation, CitationError, CitationParser, DuplicateGroup, Result};

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
/// etc.
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
/// use biblib::csv::CsvParser;
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
/// let parser = CsvParser::with_config(config);
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
    pub fn with_config(config: CsvConfig) -> Self {
        Self { config }
    }

    /// Parses a record into a Citation using the current header mapping
    fn parse_record(&self, headers: &[String], record: StringRecord) -> Result<Citation> {
        let mut citation = Citation::default();
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
                        if let Ok(year) = value.parse() {
                            citation.year = Some(year);
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
                    "label" => citation.label = Some(value.to_string()),
                    "duplicate_id" => {
                        if !value.is_empty() {
                            citation.duplicate_id = Some(value.to_string())
                        }
                    }
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

    /// Parse and group citations by duplicate ID
    pub fn parse_with_duplicates(&self, input: &str) -> Result<Vec<DuplicateGroup>> {
        let citations = self.parse(input)?;

        let mut groups: HashMap<String, Vec<Citation>> = HashMap::new();
        let mut single_uniques = Vec::new();

        // First pass - separate citations with duplicate_id from standalone uniques
        for citation in citations {
            if let Some(dup_id) = &citation.duplicate_id {
                groups.entry(dup_id.clone()).or_default().push(citation);
            } else if citation.label.as_deref() == Some("Unique") {
                // Collect unique citations that don't have a duplicate_id
                single_uniques.push(citation);
            }
        }

        // Convert groups into DuplicateGroups
        let mut result = Vec::new();

        // Process groups with duplicates
        for (_id, mut citations) in groups {
            if let Some(unique_idx) = citations
                .iter()
                .position(|c| c.label.as_deref() == Some("Unique"))
            {
                let unique = citations.remove(unique_idx);
                let duplicates = citations
                    .into_iter()
                    .filter(|c| c.label.as_deref() == Some("Duplicate"))
                    .collect();
                result.push(DuplicateGroup { unique, duplicates });
            } else {
                return Err(CitationError::MissingField(format!(
                    "No unique citation found for duplicate group {}",
                    _id
                )));
            }
        }

        // Add single unique citations as groups with empty duplicates vec
        for unique in single_uniques {
            result.push(DuplicateGroup {
                unique,
                duplicates: Vec::new(),
            });
        }

        Ok(result)
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
        assert_eq!(citations[0].year, Some(2023));
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

        let parser = CsvParser::with_config(config);
        let citations = parser.parse(input).unwrap();

        assert_eq!(citations[0].title, "Test Paper");
        assert_eq!(citations[0].authors[0].family_name, "Smith");
        assert_eq!(citations[0].year, Some(2023));
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

        let parser = CsvParser::with_config(config);
        let citations = parser.parse(input).unwrap();

        assert_eq!(citations[0].title, "Test Paper");
        assert_eq!(citations[0].authors[0].family_name, "Smith");
        assert_eq!(citations[0].year, Some(2023));
    }

    #[test]
    fn test_duplicate_groups() {
        let input = "\
Title,Author,Year,Label,DuplicateId
Original Paper,Smith J,2023,Unique,group1
Similar Paper,Smith J,2023,Duplicate,group1
Another Paper,Doe J,2022,Unique,group2
Copy Paper,Doe J,2022,Duplicate,group2";

        let parser = CsvParser::new();
        let groups = parser.parse_with_duplicates(input).unwrap();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].unique.label, Some("Unique".to_string()));
        assert_eq!(groups[0].duplicates.len(), 1);
        assert_eq!(groups[0].duplicates[0].label, Some("Duplicate".to_string()));
    }

    #[test]
    fn test_duplicate_groups_with_singles() {
        let input = "\
Title,Author,Year,Label,DuplicateId
Original Paper,Smith J,2023,Unique,group1
Similar Paper,Smith J,2023,Duplicate,group1
Standalone Paper,Doe J,2022,Unique,
Another Paper,Wilson K,2021,Unique,group2
Copy Paper,Wilson K,2021,Duplicate,group2";

        let parser = CsvParser::new();
        let groups = parser.parse_with_duplicates(input).unwrap();

        assert_eq!(groups.len(), 3); // Two groups + one standalone

        // Find the standalone paper
        let standalone = groups
            .iter()
            .find(|g| g.unique.title == "Standalone Paper")
            .unwrap();
        assert_eq!(standalone.duplicates.len(), 0);

        // Check group with duplicates
        let with_duplicate = groups
            .iter()
            .find(|g| g.unique.title == "Original Paper")
            .unwrap();
        assert_eq!(with_duplicate.duplicates.len(), 1);
    }
}
