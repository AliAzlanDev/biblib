//! RIS format data structures.
//!
//! This module defines intermediate data structures used during RIS parsing.
//!
//! # Design Decision
//!
//! ## Field Processing Strategy
//! - **Priority-based**: Journal names/abbreviations use documented priority systems
//! - **First-wins**: Simple fields like title use the first valid value found
//! - **Two-pass**: DOI extraction checks dedicated fields first, then URLs
//! - **Validation**: Date parsing includes error logging for invalid formats

use crate::ris::tags::RisTag;
use crate::{Author, CitationError};
use std::collections::HashMap;

/// Structured raw data from a RIS formatted file.
#[derive(Debug, Clone)]
pub(crate) struct RawRisData {
    /// Key-value pair data from the RIS file data.
    pub(crate) data: HashMap<RisTag, Vec<String>>,
    /// Authors of the cited work.
    pub(crate) authors: Vec<Author>,
    /// Invalid lines found in the RIS file data with line number context for error reporting.
    pub(crate) ignored_lines: Vec<(usize, String)>,
}

impl RawRisData {
    /// Create a new empty RawRisData.
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
            authors: Vec::new(),
            ignored_lines: Vec::new(),
        }
    }

    /// Add a tag-value pair to the data.
    pub(crate) fn add_data(&mut self, tag: RisTag, value: String) {
        self.data.entry(tag).or_insert_with(Vec::new).push(value);
    }

    /// Add an author to the authors list.
    pub(crate) fn add_author(&mut self, author: Author) {
        self.authors.push(author);
    }

    /// Add an ignored line with context.
    pub(crate) fn add_ignored_line(&mut self, line_number: usize, line: String) {
        self.ignored_lines.push((line_number, line));
    }

    /// Get the first value for a tag, if it exists.
    pub(crate) fn get_first(&self, tag: &RisTag) -> Option<&String> {
        self.data.get(tag).and_then(|values| values.first())
    }

    /// Remove and return all values for a tag.
    pub(crate) fn remove(&mut self, tag: &RisTag) -> Option<Vec<String>> {
        self.data.remove(tag)
    }

    /// Check if the data contains any content (not just metadata).
    pub(crate) fn has_content(&self) -> bool {
        !self.data.is_empty() || !self.authors.is_empty()
    }

    /// Get the best journal name based on tag priority.
    pub(crate) fn get_best_journal(&self) -> Option<String> {
        let mut best_journal = None;
        let mut best_priority = u8::MAX;

        for (tag, values) in &self.data {
            if let Some(priority) = tag.journal_priority() {
                if priority < best_priority && !values.is_empty() {
                    best_priority = priority;
                    best_journal = values.first().cloned();
                }
            }
        }

        best_journal
    }

    /// Get the best journal abbreviation based on tag priority.
    pub(crate) fn get_best_journal_abbr(&self) -> Option<String> {
        let mut best_abbr = None;
        let mut best_priority = u8::MAX;

        for (tag, values) in &self.data {
            if let Some(priority) = tag.journal_abbr_priority() {
                if priority < best_priority && !values.is_empty() {
                    best_priority = priority;
                    best_abbr = values.first().cloned();
                }
            }
        }

        best_abbr
    }
}

impl TryFrom<RawRisData> for crate::Citation {
    type Error = CitationError;

    fn try_from(mut raw: RawRisData) -> Result<Self, Self::Error> {
        let citation_type = raw.remove(&RisTag::Type).unwrap_or_else(Vec::new);

        let title = raw
            .get_first(&RisTag::Title)
            .or_else(|| raw.get_first(&RisTag::TitleAlternative))
            .cloned()
            .ok_or_else(|| CitationError::MissingField("title".to_string()))?;

        // Remove title data after extraction
        raw.remove(&RisTag::Title);
        raw.remove(&RisTag::TitleAlternative);

        let journal = raw.get_best_journal();
        let journal_abbr = raw.get_best_journal_abbr();

        // Clean up journal data
        raw.remove(&RisTag::JournalFull);
        raw.remove(&RisTag::JournalFullAlternative);
        raw.remove(&RisTag::JournalAbbreviation);
        raw.remove(&RisTag::JournalAbbreviationAlternative);
        raw.remove(&RisTag::SecondaryTitle);

        // Parse date from available date fields with validation
        let date = raw
            .get_first(&RisTag::PublicationYear)
            .or_else(|| raw.get_first(&RisTag::DatePrimary))
            .and_then(|date_str| {
                crate::utils::parse_ris_date(date_str)
                // Note: Invalid dates are silently ignored to avoid breaking parsing
                // TODO: Collect warnings
            });

        raw.remove(&RisTag::PublicationYear);
        raw.remove(&RisTag::DatePrimary);
        raw.remove(&RisTag::DateAccess);

        let volume = raw
            .remove(&RisTag::Volume)
            .and_then(|v| v.into_iter().next());
        let issue = raw
            .remove(&RisTag::Issue)
            .and_then(|v| v.into_iter().next());

        // Handle pages
        let start_page = raw
            .remove(&RisTag::StartPage)
            .and_then(|v| v.into_iter().next());
        let end_page = raw
            .remove(&RisTag::EndPage)
            .and_then(|v| v.into_iter().next());
        let pages = match (start_page, end_page) {
            (Some(start), Some(end)) => Some(crate::utils::format_page_numbers(&format!(
                "{}-{}",
                start, end
            ))),
            (Some(start), None) => Some(crate::utils::format_page_numbers(&start)),
            (None, Some(end)) => Some(end),
            (None, None) => None,
        };

        // First pass: Extract DOI from dedicated DOI field
        let mut doi = raw
            .remove(&RisTag::Doi)
            .and_then(|v| v.into_iter().next())
            .and_then(|doi_str| crate::utils::format_doi(&doi_str));

        let pmid = raw
            .remove(&RisTag::ReferenceId)
            .and_then(|v| v.into_iter().next());

        let pmc_id = raw
            .remove(&RisTag::PmcId)
            .and_then(|v| v.into_iter().next())
            .filter(|s| s.contains("PMC"));

        let abstract_text = raw
            .get_first(&RisTag::Abstract)
            .or_else(|| raw.get_first(&RisTag::AbstractAlternative))
            .cloned();

        raw.remove(&RisTag::Abstract);
        raw.remove(&RisTag::AbstractAlternative);

        let keywords = raw.remove(&RisTag::Keywords).unwrap_or_else(Vec::new);

        let issn = raw.remove(&RisTag::SerialNumber).unwrap_or_else(Vec::new);

        // Collect URLs from various link fields and extract DOI if not already found
        let mut urls = Vec::new();
        for tag in [
            RisTag::LinkPdf,
            RisTag::LinkFullText,
            RisTag::LinkRelated,
            RisTag::LinkImages,
            RisTag::Url,
            RisTag::Link,
        ] {
            if let Some(mut tag_urls) = raw.remove(&tag) {
                // Second pass: Extract DOI from URL fields if not already found
                if doi.is_none() {
                    for url in &tag_urls {
                        if url.contains("doi.org") {
                            if let Some(extracted_doi) = crate::utils::format_doi(url) {
                                doi = Some(extracted_doi);
                                break;
                            }
                        }
                    }
                }
                urls.append(&mut tag_urls);
            }
        }

        let language = raw
            .remove(&RisTag::Language)
            .and_then(|v| v.into_iter().next());
        let publisher = raw
            .remove(&RisTag::Publisher)
            .and_then(|v| v.into_iter().next());

        // Remove end-of-reference marker
        raw.remove(&RisTag::EndOfReference);

        // Collect remaining fields as extra_fields
        let extra_fields = raw
            .data
            .into_iter()
            .map(|(tag, values)| (tag.as_tag().to_string(), values))
            .collect();

        Ok(crate::Citation {
            citation_type,
            title,
            authors: raw.authors,
            journal,
            journal_abbr,
            date: date.clone(),
            #[allow(deprecated)]
            year: date.as_ref().map(|d| d.year),
            volume,
            issue,
            pages,
            issn,
            doi,
            pmid,
            pmc_id,
            abstract_text,
            keywords,
            urls,
            language,
            mesh_terms: Vec::new(), // RIS doesn't typically have MeSH terms
            publisher,
            extra_fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ris::tags::RisTag;

    #[test]
    fn test_raw_ris_data_new() {
        let raw = RawRisData::new();
        assert!(raw.data.is_empty());
        assert!(raw.authors.is_empty());
        assert!(raw.ignored_lines.is_empty());
        assert!(!raw.has_content());
    }

    #[test]
    fn test_add_data() {
        let mut raw = RawRisData::new();
        raw.add_data(RisTag::Title, "Test Title".to_string());
        raw.add_data(RisTag::Title, "Another Title".to_string());

        assert_eq!(
            raw.get_first(&RisTag::Title),
            Some(&"Test Title".to_string())
        );
        assert!(raw.has_content());
    }

    #[test]
    fn test_journal_priority() {
        let mut raw = RawRisData::new();
        raw.add_data(RisTag::JournalFullAlternative, "Alt Journal".to_string());
        raw.add_data(RisTag::JournalFull, "Main Journal".to_string());
        raw.add_data(RisTag::SecondaryTitle, "Secondary".to_string());

        assert_eq!(raw.get_best_journal(), Some("Main Journal".to_string()));
    }

    #[test]
    fn test_conversion_to_citation() {
        let mut raw = RawRisData::new();
        raw.add_data(RisTag::Type, "JOUR".to_string());
        raw.add_data(RisTag::Title, "Test Article".to_string());
        raw.add_author(Author {
            family_name: "Smith".to_string(),
            given_name: "John".to_string(),
            affiliation: None,
        });

        let citation: crate::Citation = raw.try_into().unwrap();
        assert_eq!(citation.title, "Test Article");
        assert_eq!(citation.citation_type, vec!["JOUR"]);
        assert_eq!(citation.authors.len(), 1);
    }

    #[test]
    fn test_missing_title_error() {
        let raw = RawRisData::new();
        let result: Result<crate::Citation, _> = raw.try_into();
        assert!(matches!(result, Err(CitationError::MissingField(_))));
    }

    #[test]
    fn test_doi_extraction_from_urls() {
        let mut raw = RawRisData::new();
        raw.add_data(RisTag::Type, "JOUR".to_string());
        raw.add_data(RisTag::Title, "Test Article".to_string());
        // Add a DOI URL without a dedicated DOI field
        raw.add_data(RisTag::Url, "https://doi.org/10.1234/example".to_string());
        raw.add_data(RisTag::LinkPdf, "https://example.com/pdf".to_string());

        let citation: crate::Citation = raw.try_into().unwrap();
        assert_eq!(citation.doi, Some("10.1234/example".to_string()));
        assert_eq!(citation.urls.len(), 2);
        assert!(citation.urls.contains(&"https://doi.org/10.1234/example".to_string()));
    }

    #[test]
    fn test_doi_extraction_prioritizes_doi_field() {
        let mut raw = RawRisData::new();
        raw.add_data(RisTag::Type, "JOUR".to_string());
        raw.add_data(RisTag::Title, "Test Article".to_string());
        // Add both dedicated DOI field and DOI URL
        raw.add_data(RisTag::Doi, "10.5678/primary".to_string());
        raw.add_data(RisTag::Url, "https://doi.org/10.1234/secondary".to_string());

        let citation: crate::Citation = raw.try_into().unwrap();
        // Should prioritize the dedicated DOI field
        assert_eq!(citation.doi, Some("10.5678/primary".to_string()));
    }
}
