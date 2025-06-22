use crate::pubmed::author::PubmedAuthor;
use crate::pubmed::tags::PubmedTag;
use crate::utils::parse_pubmed_date;
use crate::{CitationError, Date};
use std::collections::HashMap;

/// Structured raw data from a PubMed formatted .nbib file.
pub(crate) struct RawPubmedData {
    /// Key-value pair data from the .nbib file data.
    pub(crate) data: HashMap<PubmedTag, Vec<String>>,
    /// Authors of the cited work.
    pub(crate) authors: Vec<PubmedAuthor>,
    /// Invalid lines found in the .nbib file data, which were skipped by the parser.
    pub(crate) ignored_lines: Vec<String>,
}

impl TryFrom<RawPubmedData> for crate::Citation {
    type Error = CitationError;
    fn try_from(
        RawPubmedData {
            mut data,
            authors,
            ignored_lines,
        }: RawPubmedData,
    ) -> Result<Self, Self::Error> {
        // unresolved question: what should we do if multiple values are found for
        // a field where one value is expected?
        // https://github.com/AliAzlanDev/biblib/pull/7#issuecomment-2984871452
        // current solution: join multiple values on hard-coded string " AND "
        // alternative solutions:
        let date = data
            .remove(&PubmedTag::PublicationDate)
            // multiple values ignored
            .and_then(|v| v.into_iter().next())
            .map(parse_pubmed_date_err)
            .transpose()?;

        Ok(Self {
            id: nanoid::nanoid!(),
            citation_type: data
                .remove(&PubmedTag::PublicationType)
                .unwrap_or_else(Vec::new),
            title: data
                .remove(&PubmedTag::Title)
                .and_then(join_if_some)
                .ok_or_else(|| CitationError::MissingField("title".to_string()))?,
            authors: authors.into_iter().map(|a| a.into()).collect(),
            journal: data
                .remove(&PubmedTag::FullJournalTitle)
                .and_then(join_if_some),
            journal_abbr: data
                .remove(&PubmedTag::JournalTitleAbbreviation)
                .and_then(join_if_some),
            year: date.as_ref().map(|d| d.year),
            date,
            volume: data.remove(&PubmedTag::Volume).and_then(join_if_some),
            issue: data.remove(&PubmedTag::Issue).and_then(join_if_some),
            pages: data.remove(&PubmedTag::Pagination).and_then(join_if_some),
            issn: data.remove(&PubmedTag::Issn).unwrap_or_else(Vec::new),
            doi: data
                .remove(&PubmedTag::LocationId)
                .unwrap_or_else(Vec::new)
                .into_iter()
                .filter_map(parse_doi_from_lid)
                .next(),
            pmid: data
                .remove(&PubmedTag::PubmedUniqueIdentifier)
                .and_then(join_if_some),
            pmc_id: data
                .remove(&PubmedTag::PubmedCentralIdentifier)
                .and_then(join_if_some),
            abstract_text: data.remove(&PubmedTag::Abstract).and_then(join_if_some),
            keywords: Vec::new(),
            urls: Vec::new(),
            language: data.remove(&PubmedTag::Language).and_then(join_if_some),
            mesh_terms: data.remove(&PubmedTag::MeshTerms).unwrap_or_else(Vec::new),
            publisher: data.remove(&PubmedTag::Publisher).and_then(join_if_some),
            extra_fields: data
                .into_iter()
                .map(|(k, v)| (k.as_tag().to_string(), v))
                .collect(),

            // soon to be removed, see https://github.com/AliAzlanDev/biblib/issues/9#issuecomment-2989899194
            source: None,
        })
    }
}

// FIXME when `CitationError::MultipleValues` is implemented.
// https://github.com/AliAzlanDev/biblib/pull/7#issuecomment-2989915130
fn join_if_some(v: Vec<String>) -> Option<String> {
    if v.is_empty() {
        None
    } else {
        Some(v.join(" AND "))
    }
}

/// Wraps [parse_pubmed_date] to change its types.
fn parse_pubmed_date_err<S: AsRef<str>>(date: S) -> Result<Date, CitationError> {
    let s = date.as_ref();
    parse_pubmed_date(s).ok_or_else(|| CitationError::InvalidFieldValue {
        field: "date".to_string(),
        message: format!("\"{s}\" is not a valid date in YYYY MMM D format"),
    })
}

fn parse_doi_from_lid(s: String) -> Option<String> {
    s.strip_suffix(" [doi]").map(|s| s.to_string())
}

impl From<PubmedAuthor> for crate::Author {
    fn from(PubmedAuthor { name, affiliations }: PubmedAuthor) -> Self {
        Self {
            family_name: name.last_name().to_string(),
            given_name: name.given_name().unwrap_or("").to_string(),
            affiliation: if affiliations.is_empty() {
                None
            } else {
                Some(affiliations.join(" and "))
            },
        }
    }
}
