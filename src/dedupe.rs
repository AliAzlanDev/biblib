//! Citations deduplicator implementation.
//!
//! A module for detecting duplicate academic citations. It provides robust
//! deduplication of citations based on multiple criteria including DOIs, titles, journal names,
//! and other metadata.
//!
//! ## Features
//!
//! - Flexible deduplication based on multiple citation fields
//! - Smart matching of journal names and abbreviations
//! - Support for DOI and non-DOI based citations
//! - Optional year-based grouping for improved performance
//! - Parallel processing support
//! - Unicode character handling
//! - Configurable matching thresholds
//! - Source-aware deduplication with preferences
//!
//! ## Usage
//!
//! ### Basic Deduplication
//!
//! ```rust
//! use biblib::{dedupe::Deduplicator, Citation, Author, Date};
//!
//! // Create some sample citations
//! let citations = vec![
//!     Citation {
//!         title: "Machine Learning Basics".to_string(),
//!         authors: vec![
//!             Author {
//!                 family_name: "Smith".to_string(),
//!                 given_name: "John".to_string(),
//!                 affiliation: None,
//!             }
//!         ],
//!         doi: Some("10.1234/ml.2023.001".to_string()),
//!         date: Some(Date { year: 2023, month: None, day: None }),
//!         ..Default::default()
//!     },
//!     // Duplicate citation with slightly different title
//!     Citation {
//!         title: "Machine Learning Basics.".to_string(), // Notice the period
//!         authors: vec![
//!             Author {
//!                 family_name: "Smith".to_string(),
//!                 given_name: "John".to_string(),
//!                 affiliation: None,
//!             }
//!         ],
//!         doi: Some("10.1234/ml.2023.001".to_string()),
//!         date: Some(Date { year: 2023, month: None, day: None }),
//!         ..Default::default()
//!     },
//! ];
//!
//! // Create a deduplicator with default settings
//! let deduplicator = Deduplicator::new();
//!
//! // Find duplicate citations
//! let duplicate_groups = deduplicator.find_duplicates(&citations).unwrap();
//!
//! // Process results
//! for group in duplicate_groups {
//!     println!("Original: {}", group.unique.title);
//!     for duplicate in group.duplicates {
//!         println!("  Duplicate: {}", duplicate.title);
//!     }
//! }
//! ```
//!
//! ### Deduplication with Source Preferences
//!
//! ```rust
//! use biblib::{dedupe::Deduplicator, Citation};
//!
//! let citations = vec![
//!     Citation {
//!         title: "Example Title".to_string(),
//!         doi: Some("10.1234/example".to_string()),
//!         ..Default::default()
//!     },
//!     Citation {
//!         title: "Example Title".to_string(),
//!         doi: Some("10.1234/example".to_string()),
//!         ..Default::default()
//!     },
//! ];
//!
//! // Sources corresponding to each citation
//! let sources = vec!["Embase", "PubMed"];
//!
//! let config = biblib::dedupe::DeduplicatorConfig {
//!     source_preferences: vec!["PubMed".to_string(), "Embase".to_string()],
//!     ..Default::default()
//! };
//!
//! let deduplicator = Deduplicator::new().with_config(config);
//! let duplicate_groups = deduplicator.find_duplicates_with_sources(&citations, &sources).unwrap();
//!
//! // The PubMed citation will be selected as the unique citation
//! ```
//!
//! ## Advanced Configuration
//!
//! The deduplicator can be configured with custom settings:
//!
//! ```rust
//! use biblib::dedupe::{Deduplicator, DeduplicatorConfig};
//!
//! let config = DeduplicatorConfig {
//!     group_by_year: false,     // Disable year-based grouping
//!     run_in_parallel: true,    // Enable parallel processing
//!     source_preferences: vec!["PubMed".to_string(), "CrossRef".to_string()],
//! };
//!
//! let deduplicator = Deduplicator::new().with_config(config);
//! ```
//!
//! ## Matching Criteria
//!
//! Citations are considered duplicates based on the following criteria:
//!
//! 1. With DOIs:
//!    - Matching DOIs and high title similarity (≥ 0.85)
//!    - Matching journal names or ISSNs
//!
//! 2. Without DOIs:
//!    - Very high title similarity (≥ 0.93)
//!    - Matching volume or page numbers
//!    - Matching journal names or ISSNs

use crate::regex::Regex;
use crate::{Citation, DuplicateGroup};
use std::collections::HashMap;
use std::sync::LazyLock;
use strsim::jaro;
use strsim::jaro_winkler;

const DOI_TITLE_SIMILARITY_THRESHOLD: f64 = 0.85;
const NO_DOI_TITLE_SIMILARITY_THRESHOLD: f64 = 0.93;

static UNICODE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<U\+([0-9A-Fa-f]+)>").unwrap());

const HTML_REPLACEMENTS: [(&str, &str); 13] = [
    ("&lt;", "<"),
    ("&gt;", ">"),
    ("<sup>", ""),
    ("</sup>", ""),
    ("<sub>", ""),
    ("</sub>", ""),
    ("<inf>", ""),
    ("</inf>", ""),
    ("beta", "b"),
    ("alpha", "a"),
    ("α", "a"),
    ("ß", "b"),
    ("γ", "g"),
];

/// Configuration options for controlling the deduplication process.
///
/// This struct allows fine-tuning of the deduplication algorithm's behavior
/// through various options. The main settings control grouping strategy and
/// parallel processing capabilities.
///
/// # Examples
///
/// ```
/// use biblib::dedupe::DeduplicatorConfig;
///
/// let config = DeduplicatorConfig {
///     group_by_year: true,    // Enable year-based grouping
///     run_in_parallel: true,  // Enable parallel processing
///     source_preferences: vec!["PubMed".to_string(), "Google Scholar      ".to_string()],
/// };
/// ```
///
/// # Performance Impact
///
/// - `group_by_year`: Significant performance improvement for large datasets
/// - `run_in_parallel`: Most effective when used with year grouping
///
/// # Notes
///
/// - When `group_by_year` is false, `run_in_parallel` is automatically disabled
/// - Year grouping is recommended for datasets with > 1000 citations
#[derive(Debug, Default, Clone)]
pub struct DeduplicatorConfig {
    /// Whether to group citations by year before processing.
    /// This can significantly improve performance for large datasets.
    pub group_by_year: bool,
    /// Whether to use parallel processing for year groups.
    /// Most effective when combined with `group_by_year = true`.
    pub run_in_parallel: bool,
    /// Ordered list of preferred sources for unique citations.
    /// First source in the list has highest priority.
    pub source_preferences: Vec<String>,
}

/// Core deduplication engine for finding duplicate citations.
///
/// The deduplicator uses a sophisticated algorithm to identify duplicate citations
/// based on multiple criteria including DOIs, titles, and other metadata. It supports
/// both exact and fuzzy matching with configurable thresholds.
///
/// # Algorithm
///
/// Citations are considered duplicates based on these criteria:
///
/// 1. **With DOIs**:
///    - Matching DOIs and high title similarity (≥ 0.85)
///    - Matching journal names or ISSNs
///
/// 2. **Without DOIs**:
///    - Very high title similarity (≥ 0.93)
///    - Matching volume/pages
///    - Matching journal names/ISSNs
///
/// # Examples
///
/// ```
/// use biblib::dedupe::{Deduplicator, DeduplicatorConfig};
///
/// // Create with default settings
///
/// // Or with custom configuration
/// let config = DeduplicatorConfig {
///     group_by_year: true,
///     run_in_parallel: true,
///     source_preferences: vec!["PubMed".to_string(), "Embase".to_string()],
/// };
/// let deduplicator = Deduplicator::new().with_config(config);
/// ```
///
/// # Performance
///
/// - Time complexity: O(n²) without year grouping
/// - With year grouping: O(Σ n_y²) where n_y is citations per year
/// - Parallel processing available when using year grouping
#[derive(Debug, Default, Clone)]
pub struct Deduplicator {
    config: DeduplicatorConfig,
}

#[derive(Debug)]
struct PreprocessedCitation<'a> {
    original: &'a Citation,
    normalized_title: String,
    normalized_journal: Option<String>,
    normalized_journal_abbr: Option<String>,
    normalized_issn: Vec<String>,
    normalized_volume: String,
}

/// Error types for dedupe operations
#[derive(Debug, thiserror::Error)]
pub enum DedupeError {
    #[error("Invalid citation data: {0}")]
    InvalidCitation(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl Deduplicator {
    /// Creates a new Deduplicator with default configuration.
    ///
    /// Default configuration enables year-based grouping and disables parallel processing.
    ///
    /// # Examples
    ///
    /// ```
    /// use biblib::dedupe::Deduplicator;
    ///
    /// let deduplicator = Deduplicator::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: DeduplicatorConfig {
                group_by_year: true,
                run_in_parallel: false,
                source_preferences: Vec::new(),
            },
        }
    }

    /// Creates a new Deduplicator with custom configuration.
    ///
    /// # Notes
    ///
    /// - Disabling year-based grouping can result in very long processing times.
    /// - Parallel processing (`run_in_parallel`) is only effective when `group_by_year` is `true`.
    /// - If `run_in_parallel` is `true` but `group_by_year` is `false`, `run_in_parallel` will be ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use biblib::dedupe::{Deduplicator, DeduplicatorConfig};
    ///
    /// let config = DeduplicatorConfig {
    ///     group_by_year: true,
    ///     run_in_parallel: true,
    ///     source_preferences: vec!["PubMed".to_string(), "Google Scholar".to_string()],   
    /// };
    /// let deduplicator = Deduplicator::new().with_config(config);
    /// ```
    #[must_use]
    pub fn with_config(mut self, mut config: DeduplicatorConfig) -> Self {
        // Disable parallel processing if not grouping by year
        if !config.group_by_year {
            config.run_in_parallel = false;
        }
        self.config = config;
        self
    }

    /// Processes a list of citations and returns groups of duplicates.
    ///
    /// This method analyzes the provided citations and groups them based on
    /// similarity criteria including DOIs, titles, and other metadata.
    /// One citation in each group is designated as the unique (original) citation.
    ///
    /// # Arguments
    ///
    /// * `citations` - A slice of Citation objects to be analyzed
    ///
    /// # Returns
    ///
    /// Returns a vector of `DuplicateGroup`s, where each group contains
    /// one unique citation and its identified duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use biblib::{dedupe::Deduplicator, Citation};
    ///
    /// let citations = vec![
    ///     Citation {
    ///         title: "Example Title".to_string(),
    ///         doi: Some("10.1234/example".to_string()),
    ///         ..Default::default()
    ///     },
    ///     // ... more citations ...
    /// ];
    ///
    /// let deduplicator = Deduplicator::new();
    /// let duplicate_groups = deduplicator.find_duplicates(&citations).unwrap();
    /// ```
    pub fn find_duplicates(
        self,
        citations: &[Citation],
    ) -> Result<Vec<DuplicateGroup>, DedupeError> {
        self.find_duplicates_with_sources(citations, &[])
    }

    /// Processes citations with their source information and returns groups of duplicates.
    ///
    /// This method is similar to `find_duplicates` but allows you to specify source
    /// information for each citation, enabling source-based preferences during deduplication.
    /// Citations without corresponding source entries are treated as having no source.
    ///
    /// # Arguments
    ///
    /// * `citations` - A slice of Citation objects to be analyzed
    /// * `sources` - A slice of source names corresponding to each citation.
    ///   If shorter than citations, remaining citations have no source.
    ///
    /// # Returns
    ///
    /// Returns a vector of `DuplicateGroup`s, where each group contains
    /// one unique citation and its identified duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use biblib::{dedupe::Deduplicator, Citation};
    ///
    /// let citations = vec![
    ///     Citation {
    ///         title: "Example Title".to_string(),
    ///         doi: Some("10.1234/example".to_string()),
    ///         ..Default::default()
    ///     },
    ///     Citation {
    ///         title: "Example Title".to_string(),
    ///         doi: Some("10.1234/example".to_string()),
    ///         ..Default::default()
    ///     },
    /// ];
    ///
    /// let sources = vec!["PubMed", "CrossRef"];
    ///
    /// let deduplicator = Deduplicator::new();
    /// let duplicate_groups = deduplicator.find_duplicates_with_sources(&citations, &sources).unwrap();
    /// ```
    pub fn find_duplicates_with_sources(
        self,
        citations: &[Citation],
        sources: &[&str],
    ) -> Result<Vec<DuplicateGroup>, DedupeError> {
        if citations.is_empty() {
            return Ok(Vec::new());
        }

        // Validate input - warn if sources length exceeds citations
        if sources.len() > citations.len() {
            return Err(DedupeError::ConfigError(format!(
                "Number of sources ({}) exceeds number of citations ({}). Each source must correspond to a citation.",
                sources.len(),
                citations.len()
            )));
        }

        // Create source mapping using citation indices instead of IDs
        let source_map: HashMap<usize, Option<&str>> = citations
            .iter()
            .enumerate()
            .zip(
                sources
                    .iter()
                    .map(|&s| Some(s))
                    .chain(std::iter::repeat(None)),
            )
            .map(|((idx, _citation), source)| (idx, source))
            .collect();

        if self.config.group_by_year {
            let year_groups = Self::group_by_year(citations);
            if self.config.run_in_parallel {
                use rayon::prelude::*;

                let duplicate_groups: Result<Vec<_>, _> = year_groups
                    .par_iter()
                    .map(|(_, citations_in_year)| {
                        self.process_citation_group_with_sources(citations_in_year, &source_map)
                    })
                    .collect();

                // Flatten results
                Ok(duplicate_groups?.into_iter().flatten().collect())
            } else {
                let mut duplicate_groups = Vec::new();

                for citations_in_year in year_groups.values() {
                    duplicate_groups.extend(
                        self.process_citation_group_with_sources(citations_in_year, &source_map)?,
                    );
                }
                Ok(duplicate_groups)
            }
        } else {
            let citations_refs: Vec<&Citation> = citations.iter().collect();
            self.process_citation_group_with_sources(&citations_refs, &source_map)
        }
    }

    /// Get the year from a citation, providing backward compatibility.
    /// Prefers the new `date.year` field, falls back to deprecated `year` field.
    fn get_citation_year(citation: &Citation) -> Option<i32> {
        Self::get_citation_year_static(citation)
    }

    fn select_unique_citation<'a>(&self, citations: &[&'a Citation]) -> &'a Citation {
        if citations.len() == 1 {
            return citations[0];
        }

        // If no source preference matches, prefer citations with abstracts
        let citations_with_abstract: Vec<_> = citations
            .iter()
            .filter(|c| c.abstract_text.is_some())
            .collect();

        match citations_with_abstract.len() {
            0 => citations[0],               // If no abstracts, use first citation
            1 => citations_with_abstract[0], // If one abstract, use that
            _ => {
                // Multiple abstracts, prefer ones with DOI
                let with_doi = citations_with_abstract
                    .iter()
                    .find(|c| c.doi.as_ref().is_some_and(|d| !d.is_empty()));

                with_doi.copied().unwrap_or(citations_with_abstract[0])
            }
        }
    }

    fn select_unique_citation_with_sources<'a>(
        &self,
        citations: &[&'a Citation],
        citation_indices: &[usize],
        source_map: &HashMap<usize, Option<&str>>,
    ) -> &'a Citation {
        if citations.len() == 1 {
            return citations[0];
        }

        // First try source preferences
        if !self.config.source_preferences.is_empty() {
            for preferred_source in &self.config.source_preferences {
                for (citation, &idx) in citations.iter().zip(citation_indices.iter()) {
                    if source_map.get(&idx) == Some(&Some(preferred_source.as_str())) {
                        return citation;
                    }
                }
            }
        }

        // If no source preference matches, use the standard selection logic
        self.select_unique_citation(citations)
    }

    fn process_citation_group_with_sources(
        &self,
        citations: &[&Citation],
        source_map: &HashMap<usize, Option<&str>>,
    ) -> Result<Vec<DuplicateGroup>, DedupeError> {
        let mut duplicate_groups = Vec::new();

        // Create a mapping from citation pointer to original index for source lookup
        let citation_to_index: HashMap<*const Citation, usize> = citations
            .iter()
            .enumerate()
            .map(|(i, citation)| (*citation as *const Citation, i))
            .collect();

        // Preprocess all citations in this group
        let preprocessed: Vec<PreprocessedCitation> = citations
            .iter()
            .map(|c| {
                Ok(PreprocessedCitation {
                    original: c,
                    normalized_title: Self::normalize_string(&Self::convert_unicode_string(
                        &c.title,
                    ))
                    .ok_or_else(|| {
                        DedupeError::ProcessingError("Failed to normalize title".to_string())
                    })?,
                    normalized_journal: Self::format_journal_name(c.journal.as_deref()),
                    normalized_journal_abbr: Self::format_journal_name(c.journal_abbr.as_deref()),
                    normalized_volume: c
                        .volume
                        .as_deref()
                        .map_or(String::new(), Deduplicator::normalize_volume),
                    normalized_issn: c
                        .issn
                        .iter()
                        .filter_map(|issn| Deduplicator::format_issn(issn))
                        .collect(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut processed_indices = std::collections::HashSet::new();

        for i in 0..preprocessed.len() {
            if processed_indices.contains(&i) {
                continue;
            }

            let mut group_citations = vec![preprocessed[i].original];
            let mut group_indices = vec![i];
            let current = &preprocessed[i];

            for (j, other) in preprocessed.iter().enumerate() {
                if i == j || processed_indices.contains(&j) {
                    continue;
                }

                let journal_match = Self::journals_match(
                    &current.normalized_journal,
                    &current.normalized_journal_abbr,
                    &other.normalized_journal,
                    &other.normalized_journal_abbr,
                );
                let issns_match =
                    Self::match_issns(&current.normalized_issn, &other.normalized_issn);
                let volumes_match = !current.normalized_volume.is_empty()
                    && !other.normalized_volume.is_empty()
                    && current.normalized_volume == other.normalized_volume;
                let pages_match = current.original.pages.is_some()
                    && other.original.pages.is_some()
                    && current.original.pages == other.original.pages;
                let years_match = Self::get_citation_year(current.original)
                    == Self::get_citation_year(other.original);

                let is_duplicate = match (&current.original.doi, &other.original.doi) {
                    // With DOIs
                    (Some(doi1), Some(doi2)) if !doi1.is_empty() && !doi2.is_empty() => {
                        let title_similarity =
                            jaro(&current.normalized_title, &other.normalized_title);

                        // With Journal/ISSN match
                        (doi1 == doi2 && title_similarity >= DOI_TITLE_SIMILARITY_THRESHOLD && (journal_match || issns_match))
                        // Without Journal/ISSN match: only when we have same DOI (and we use volume/pages instead)
                        || (doi1 == doi2 && title_similarity >= 0.99 && (volumes_match || pages_match))
                        // Without DOI match: only when we have a very high title similarity and all other fields match
                        || (title_similarity >= 0.99 && years_match && (volumes_match || pages_match) && (journal_match || issns_match))
                    }
                    // Without DOIs
                    _ => {
                        let title_similarity =
                            jaro_winkler(&current.normalized_title, &other.normalized_title);

                        // With Journal/ISSN match
                        (title_similarity >= NO_DOI_TITLE_SIMILARITY_THRESHOLD && (volumes_match || pages_match) && (journal_match || issns_match))
                        // Without Journal/ISSN match: only when we have a very high title similarity and all other fields match
                        || (title_similarity >= 0.99 && years_match && (volumes_match && pages_match))
                    }
                };

                if is_duplicate {
                    group_citations.push(other.original);
                    group_indices.push(j);
                    processed_indices.insert(j);
                }
            }

            if group_citations.len() > 1 {
                // Convert citation indices to original indices for source lookup
                let original_indices: Vec<usize> = group_indices
                    .iter()
                    .map(|&local_idx| {
                        let citation_ptr = preprocessed[local_idx].original as *const Citation;
                        citation_to_index[&citation_ptr]
                    })
                    .collect();

                let unique = self.select_unique_citation_with_sources(
                    &group_citations,
                    &original_indices,
                    source_map,
                );

                let duplicates: Vec<Citation> = group_citations
                    .into_iter()
                    .filter(|c| !std::ptr::eq(*c, unique))
                    .map(|c| (*c).clone())
                    .collect();

                duplicate_groups.push(DuplicateGroup {
                    unique: unique.clone(),
                    duplicates,
                });
                processed_indices.insert(i);
            } else {
                duplicate_groups.push(DuplicateGroup {
                    unique: current.original.clone(),
                    duplicates: Vec::new(),
                });
            }
        }

        Ok(duplicate_groups)
    }

    fn group_by_year(citations: &[Citation]) -> HashMap<i32, Vec<&Citation>> {
        let mut year_map: HashMap<i32, Vec<&Citation>> = HashMap::new();

        // TODO: handle citations without a year when grouping by year
        for citation in citations {
            let year = Self::get_citation_year_static(citation).unwrap_or(0);
            year_map.entry(year).or_default().push(citation);
        }

        year_map
    }
    /// Static version of get_citation_year for use in static contexts
    fn get_citation_year_static(citation: &Citation) -> Option<i32> {
        #[allow(deprecated)]
        citation.date.as_ref().map(|d| d.year).or(citation.year)
    }

    fn convert_unicode_string(input: &str) -> String {
        UNICODE_REGEX
            .replace_all(input, |caps: &crate::regex::Captures| {
                u32::from_str_radix(&caps[1], 16)
                    .ok()
                    .and_then(char::from_u32)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| caps[0].to_string())
            })
            .to_string()
    }

    fn normalize_string(string: &str) -> Option<String> {
        if string.is_empty() {
            return None;
        }

        let mut result = String::with_capacity(string.len());
        let mut s = string.trim().to_lowercase();

        for replacement in HTML_REPLACEMENTS.iter() {
            s = s.replace(replacement.0, replacement.1);
        }

        s.chars()
            .filter(|c| c.is_alphanumeric())
            .for_each(|c| result.push(c));

        Some(result)
    }

    fn normalize_volume(volume: &str) -> String {
        if volume.is_empty() {
            return String::new();
        }

        // Find first sequence of numbers anywhere in the string
        let numbers: String = volume
            .chars()
            .skip_while(|c| !c.is_numeric())
            .take_while(|c| c.is_numeric())
            .collect();

        if numbers.is_empty() {
            String::new()
        } else {
            numbers
        }
    }

    /// Check if two journals match by comparing both full name and abbreviation
    fn journals_match(
        journal1: &Option<String>,
        journal_abbr1: &Option<String>,
        journal2: &Option<String>,
        journal_abbr2: &Option<String>,
    ) -> bool {
        journal1
            .as_ref()
            .zip(journal2.as_ref())
            .is_some_and(|(j1, j2)| j1 == j2)
            || journal_abbr1
                .as_ref()
                .zip(journal_abbr2.as_ref())
                .is_some_and(|(a1, a2)| a1 == a2)
            || journal1
                .as_ref()
                .zip(journal_abbr2.as_ref())
                .is_some_and(|(j1, a2)| j1 == a2)
            || journal_abbr1
                .as_ref()
                .zip(journal2.as_ref())
                .is_some_and(|(a1, j2)| a1 == j2)
    }

    fn format_journal_name(full_name: Option<&str>) -> Option<String> {
        full_name.map(|name| {
            name.split(". Conference")
                .next()
                .unwrap_or(name)
                .trim()
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
    }

    fn format_issn(issn_str: &str) -> Option<String> {
        // Remove common suffixes and extra text
        let clean_issn = issn_str
            .trim()
            .replace("(Electronic)", "")
            .replace("(Linking)", "")
            .replace("(Print)", "")
            .replace(|c: char| !c.is_ascii_digit() && c != '-' && c != 'X', "")
            .trim()
            .to_string();

        // Extract all digits and X
        let digits: String = clean_issn
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == 'X')
            .collect();

        // Validate format
        match (clean_issn.len(), digits.len()) {
            // Valid formats: "1234-5678" (9 chars with hyphen) or "12345678" (8 chars without hyphen)
            (9, 8) if clean_issn.chars().nth(4) == Some('-') => Some(clean_issn),
            (8, 8) => Some(format!("{}-{}", &digits[..4], &digits[4..])),
            _ => None,
        }
    }

    fn match_issns(list1: &[String], list2: &[String]) -> bool {
        list1
            .iter()
            .any(|isbn1| list2.iter().any(|isbn2| isbn1 == isbn2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_by_year() {
        let citations = vec![
            Citation {
                title: "Title 1".to_string(),
                authors: vec![],
                journal: None,
                journal_abbr: None,
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                volume: None,
                abstract_text: None,
                doi: None,
                ..Default::default()
            },
            Citation {
                title: "Title 2".to_string(),
                authors: vec![],
                journal: None,
                journal_abbr: None,
                date: None,
                volume: None,
                abstract_text: None,
                doi: None,
                ..Default::default()
            },
        ];

        let grouped = Deduplicator::group_by_year(&citations);
        assert_eq!(grouped.get(&2020).unwrap().len(), 1);
        assert_eq!(grouped.get(&0).unwrap().len(), 1);
    }

    #[test]
    fn test_find_duplicates() {
        let citations = vec![
            Citation {
                title: "Title 1".to_string(),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                doi: Some("10.1234/abc".to_string()),
                journal: Some("Journal 1".to_string()),
                ..Default::default()
            },
            Citation {
                title: "Title 1".to_string(),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                doi: Some("10.1234/abc".to_string()),
                journal: Some("Journal 1".to_string()),
                ..Default::default()
            },
            Citation {
                title: "Title 2".to_string(),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                doi: Some("10.1234/def".to_string()),
                journal: Some("Journal 2".to_string()),
                ..Default::default()
            },
        ];

        let deduplicator = Deduplicator::new();
        let duplicate_groups = deduplicator.find_duplicates(&citations).unwrap();

        assert_eq!(duplicate_groups.len(), 2);
        assert_eq!(
            duplicate_groups
                .iter()
                .find(|g| g.unique.doi == Some("10.1234/abc".to_string()))
                .unwrap()
                .duplicates
                .len(),
            1
        );
    }

    #[test]
    fn test_missing_doi() {
        let citations = vec![
            Citation {
                title: "Title 1".to_string(),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                doi: Some("10.1234/abc".to_string()),
                journal: Some("Journal 1".to_string()),
                volume: Some("24".to_string()),
                ..Default::default()
            },
            Citation {
                title: "Title 1".to_string(),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                doi: Some("".to_string()),
                journal: Some("Journal 1".to_string()),
                volume: Some("24".to_string()),
                ..Default::default()
            },
            Citation {
                title: "Title 2".to_string(),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                doi: Some("".to_string()),
                journal: Some("Journal 2".to_string()),
                ..Default::default()
            },
        ];

        let deduplicator = Deduplicator::new();
        let duplicate_groups = deduplicator.find_duplicates(&citations).unwrap();

        assert_eq!(duplicate_groups.len(), 2);
    }

    #[test]
    fn test_normalize_string() {
        assert_eq!(
            Deduplicator::normalize_string("Machine Learning! (2<sup>nd</sup> Edition)"),
            Some("machinelearning2ndedition".to_string())
        );
        assert_eq!(
            Deduplicator::normalize_string("[&lt;sup&gt;11&lt;/sup&gt;C] benzo"),
            Some("11cbenzo".to_string())
        );
    }

    #[test]
    fn test_convert_unicode_string() {
        // Test basic conversion
        assert_eq!(
            Deduplicator::convert_unicode_string("2<U+0391>-amino-4<U+0391>"),
            "2Α-amino-4Α",
            "Failed to convert basic Alpha Unicode sequences"
        );

        // Test multiple different Unicode sequences
        assert_eq!(
            Deduplicator::convert_unicode_string("Hello <U+03A9>orld <U+03A3>cience"),
            "Hello Ωorld Σcience",
            "Failed to convert multiple Unicode sequences"
        );

        // Test string with no Unicode sequences
        assert_eq!(
            Deduplicator::convert_unicode_string("Normal String"),
            "Normal String",
            "Incorrectly modified string with no Unicode sequences"
        );

        // Test empty string
        assert_eq!(
            Deduplicator::convert_unicode_string(""),
            "",
            "Failed to handle empty string"
        );

        // Test mixed content
        assert_eq!(
            Deduplicator::convert_unicode_string("Mixed <U+0394> Unicode <U+03A9> Test"),
            "Mixed Δ Unicode Ω Test",
            "Failed to handle mixed content with Unicode sequences"
        );

        // Test consecutive Unicode sequences
        assert_eq!(
            Deduplicator::convert_unicode_string("<U+0391><U+0392><U+0393>"),
            "ΑΒΓ",
            "Failed to convert consecutive Unicode sequences"
        );
    }

    #[test]
    fn test_normalize_volume() {
        assert_eq!(Deduplicator::normalize_volume("61"), "61");
        assert_eq!(Deduplicator::normalize_volume("61 (Supplement 1)"), "61");
        assert_eq!(Deduplicator::normalize_volume("9 (8) (no pagination)"), "9");
        assert_eq!(Deduplicator::normalize_volume("3)"), "3");
        assert_eq!(Deduplicator::normalize_volume("Part A. 242"), "242");
        assert_eq!(Deduplicator::normalize_volume("55 (10 SUPPL 1)"), "55");
        assert_eq!(Deduplicator::normalize_volume("161A"), "161");
        assert_eq!(Deduplicator::normalize_volume("74 Suppl 1"), "74");
        assert_eq!(Deduplicator::normalize_volume("20 (2)"), "20");
        assert_eq!(
            Deduplicator::normalize_volume("9 (FEB) (no pagination)"),
            "9"
        );
    }

    #[test]
    fn test_format_journal_name() {
        assert_eq!(
            Deduplicator::format_journal_name(Some(
                "Heart. Conference: British Atherosclerosis Society BAS/British Society for Cardiovascular Research BSCR Annual Meeting"
            )),
            Some("heart".to_string())
        );
        assert_eq!(
            Deduplicator::format_journal_name(Some(
                "The FASEB Journal. Conference: Experimental Biology"
            )),
            Some("thefasebjournal".to_string())
        );
        assert_eq!(
            Deduplicator::format_journal_name(Some(
                "Arteriosclerosis Thrombosis and Vascular Biology. Conference: American Heart Association's Arteriosclerosis Thrombosis and Vascular Biology"
            )),
            Some("arteriosclerosisthrombosisandvascularbiology".to_string())
        );
        assert_eq!(Deduplicator::format_journal_name(None), None);
        assert_eq!(
            Deduplicator::format_journal_name(Some("")),
            Some("".to_string())
        );
        assert_eq!(
            Deduplicator::format_journal_name(Some("Diabetologie und Stoffwechsel. Conference")),
            Some("diabetologieundstoffwechsel".to_string())
        );
    }

    #[test]
    fn test_match_issns_scenarios() {
        // Scenario 1: Matching lists
        let issns1 = vec!["1234-5678".to_string(), "8765-4321".to_string()];
        let issns2 = vec!["0000-0000".to_string(), "1234-5678".to_string()];
        assert!(
            Deduplicator::match_issns(&issns1, &issns2),
            "Should find a matching ISSN"
        );

        let non_match_issns2 = vec!["5555-6666".to_string(), "7777-8888".to_string()];
        assert!(
            !Deduplicator::match_issns(&issns1, &non_match_issns2),
            "Should not find a matching ISSN"
        );

        // Scenario 3: Empty lists
        let empty_issns1: Vec<String> = vec![];
        let empty_issns2: Vec<String> = vec![];
        assert!(
            !Deduplicator::match_issns(&empty_issns1, &empty_issns2),
            "Should return false for empty lists"
        );

        // Scenario 4: One empty list
        let partial_issns1 = vec!["1234-5678".to_string()];
        let partial_issns2: Vec<String> = vec![];
        assert!(
            !Deduplicator::match_issns(&partial_issns1, &partial_issns2),
            "Should return false when one list is empty"
        );
    }

    #[test]
    fn test_format_issn() {
        assert_eq!(
            Deduplicator::format_issn("1234-5678"),
            Some("1234-5678".to_string())
        );
        assert_eq!(
            Deduplicator::format_issn("12345678"),
            Some("1234-5678".to_string())
        );
        assert_eq!(
            Deduplicator::format_issn("1234-567X"),
            Some("1234-567X".to_string())
        );
        assert_eq!(
            Deduplicator::format_issn("1234-567X (Electronic)"),
            Some("1234-567X".to_string())
        );
        assert_eq!(
            Deduplicator::format_issn("1234-5678 (Print)"),
            Some("1234-5678".to_string())
        );
        assert_eq!(
            Deduplicator::format_issn("1234-5678 (Linking)"),
            Some("1234-5678".to_string())
        );
        assert_eq!(Deduplicator::format_issn("invalid"), None);
        assert_eq!(Deduplicator::format_issn("1234-56789"), None);
        assert_eq!(Deduplicator::format_issn("123-45678"), None);
    }

    #[test]
    fn test_without_year_grouping() {
        let citations = vec![
            Citation {
                title: "Title 1".to_string(),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                doi: Some("10.1234/abc".to_string()),
                journal: Some("Journal 1".to_string()),
                ..Default::default()
            },
            Citation {
                title: "Title 1".to_string(),
                date: Some(crate::Date {
                    year: 2019, // Different year
                    month: None,
                    day: None,
                }),
                doi: Some("10.1234/abc".to_string()),
                journal: Some("Journal 1".to_string()),
                ..Default::default()
            },
        ];

        let config = DeduplicatorConfig {
            group_by_year: false,
            ..Default::default()
        };
        let deduplicator = Deduplicator::new().with_config(config);
        let duplicate_groups = deduplicator.find_duplicates(&citations).unwrap();

        assert_eq!(duplicate_groups.len(), 1);
        assert_eq!(duplicate_groups[0].duplicates.len(), 1);

        // Test with default year grouping (should not find duplicates across years)
        let deduplicator = Deduplicator::new();
        let duplicate_groups = deduplicator.find_duplicates(&citations).unwrap();

        assert_eq!(duplicate_groups.len(), 2);
        assert!(duplicate_groups.iter().all(|g| g.duplicates.is_empty()));
    }

    #[test]
    fn test_source_preferences() {
        let citations = vec![
            Citation {
                title: "Title 1".to_string(),
                doi: Some("10.1234/abc".to_string()),
                journal: Some("Journal 1".to_string()),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                ..Default::default()
            },
            Citation {
                title: "Title 1".to_string(),
                doi: Some("10.1234/abc".to_string()),
                journal: Some("Journal 1".to_string()),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                ..Default::default()
            },
        ];

        let sources = vec!["source2", "source1"];

        let config = DeduplicatorConfig {
            source_preferences: vec!["source1".to_string(), "source2".to_string()],
            ..Default::default()
        };

        let deduplicator = Deduplicator::new().with_config(config);
        let duplicate_groups = deduplicator
            .find_duplicates_with_sources(&citations, &sources)
            .unwrap();

        assert_eq!(duplicate_groups.len(), 1);
        // The second citation should be selected as unique because source1 (PubMed)
        // has higher priority than source2 (Embase) in our preferences
        assert_eq!(duplicate_groups[0].duplicates.len(), 1);
    }

    #[test]
    fn test_abstract_preference() {
        let citations = vec![
            Citation {
                title: "Title 1".to_string(),
                abstract_text: None,
                doi: Some("10.1234/abc".to_string()),
                journal: Some("Journal 1".to_string()),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                ..Default::default()
            },
            Citation {
                title: "Title 1".to_string(),
                abstract_text: Some("Abstract".to_string()),
                doi: Some("10.1234/abc".to_string()),
                journal: Some("Journal 1".to_string()),
                date: Some(crate::Date {
                    year: 2020,
                    month: None,
                    day: None,
                }),
                ..Default::default()
            },
        ];

        let deduplicator = Deduplicator::new();
        let duplicate_groups = deduplicator.find_duplicates(&citations).unwrap();

        assert_eq!(duplicate_groups.len(), 1);
        // The citation with abstract should be selected as unique
        assert!(duplicate_groups[0].unique.abstract_text.is_some());
        assert_eq!(duplicate_groups[0].duplicates.len(), 1);
    }
}
