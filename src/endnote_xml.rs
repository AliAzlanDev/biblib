//! EndNote XML format parser implementation with source tracking support.
//!
//! Provides functionality to parse EndNote XML formatted citations with built-in source tracking.
//!
//! # Example
//!
//! ```
//! use biblib::{CitationParser, EndNoteXmlParser};
//!
//! let input = r#"<?xml version="1.0" encoding="UTF-8"?>
//! <xml><records><record>
//! <titles><title>Example Title</title></titles>
//! <contributors><authors><author>Smith, John</author></authors></contributors>
//! </record></records></xml>"#;
//!
//! let parser = EndNoteXmlParser::new()
//!     .with_source("Embase");
//!     
//! let citations = parser.parse(input).unwrap();
//! assert_eq!(citations[0].title, "Example Title");
//! assert_eq!(citations[0].source.clone().unwrap(), "Embase");
//! ```

use nanoid::nanoid;
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use std::io::BufRead;

use crate::utils::{format_doi, format_page_numbers, parse_author_name, split_issns};
use crate::{Author, Citation, CitationError, CitationParser, Result};

/// Parser for EndNote XML format citations.
#[derive(Debug, Default, Clone)]
pub struct EndNoteXmlParser {
    source: Option<String>,
}

impl EndNoteXmlParser {
    /// Creates a new EndNote XML parser instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use biblib::EndNoteXmlParser;
    /// let parser = EndNoteXmlParser::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { source: None }
    }

    #[must_use]
    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// Extracts text content from XML events until the closing tag is found
    fn extract_text<B: BufRead>(
        reader: &mut Reader<B>,
        buf: &mut Vec<u8>,
        closing_tag: &[u8],
    ) -> Result<String> {
        let mut text = String::new();
        let closing_tag_str = String::from_utf8_lossy(closing_tag);

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Text(e)) => {
                    text.push_str(&e.unescape().map_err(|e| {
                        CitationError::InvalidFormat(format!("Invalid XML text content: {}", e))
                    })?);
                }
                Ok(Event::End(e)) if e.name() == QName(closing_tag) => break,
                Ok(Event::Eof) => {
                    return Err(CitationError::InvalidFormat(format!(
                        "Unexpected EOF while looking for closing tag '{}'",
                        closing_tag_str
                    )))
                }
                Err(e) => return Err(CitationError::from(e)),
                _ => continue,
            }
            buf.clear();
        }

        Ok(text.trim().to_string())
    }

    /// Parse a single record element into a Citation
    fn parse_record<B: BufRead>(
        &self,
        reader: &mut Reader<B>,
        buf: &mut Vec<u8>,
    ) -> Result<Citation> {
        let mut citation = Citation::default();
        citation.id = nanoid!();
        citation.citation_type.push("Journal Article".to_string()); // Set default type
        citation.source = self.source.clone(); // Now we can access self.source

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"ref-type" => {
                        citation.citation_type.clear(); // Clear default before adding new type
                        for attr in e.attributes() {
                            let attr = attr.map_err(CitationError::from)?;
                            if attr.key.as_ref() == b"name" {
                                citation.citation_type.push(
                                    attr.unescape_value()
                                        .map_err(CitationError::from)?
                                        .into_owned(),
                                );
                            }
                        }
                    }
                    b"title" => {
                        citation.title = Self::extract_text(reader, buf, b"title")?;
                    }
                    b"author" => {
                        let author_str = Self::extract_text(reader, buf, b"author")?;
                        let (family, given) = parse_author_name(&author_str);
                        citation.authors.push(Author {
                            family_name: family,
                            given_name: given,
                            affiliation: None,
                        });
                    }
                    b"secondary-title" => {
                        citation.journal =
                            Some(Self::extract_text(reader, buf, b"secondary-title")?);
                    }
                    b"alt-title" => {
                        citation.journal_abbr =
                            Some(Self::extract_text(reader, buf, b"alt-title")?);
                    }
                    b"custom2" => {
                        let text = Self::extract_text(reader, buf, b"custom2")?;
                        if text.contains("PMC") {
                            citation.pmc_id = Some(text);
                        }
                    }
                    b"volume" => {
                        citation.volume = Some(Self::extract_text(reader, buf, b"volume")?);
                    }
                    b"number" => {
                        citation.issue = Some(Self::extract_text(reader, buf, b"number")?);
                    }
                    b"pages" => {
                        citation.pages = Some(format_page_numbers(&Self::extract_text(
                            reader, buf, b"pages",
                        )?));
                    }
                    b"electronic-resource-num" => {
                        let doi = Self::extract_text(reader, buf, b"electronic-resource-num")?;
                        if doi.starts_with("10.") || doi.contains("doi.org") {
                            citation.doi = format_doi(&doi);
                        }
                    }
                    b"url" => {
                        let url = Self::extract_text(reader, buf, b"url")?;
                        if citation.doi.is_none() && url.contains("doi.org") {
                            citation.doi = format_doi(&url);
                        }
                        citation.urls.push(url);
                    }
                    b"year" => {
                        if let Ok(year) = Self::extract_text(reader, buf, b"year")?.parse::<i32>() {
                            citation.year = Some(year);
                        }
                    }
                    b"abstract" => {
                        citation.abstract_text =
                            Some(Self::extract_text(reader, buf, b"abstract")?);
                    }
                    b"keyword" => {
                        citation
                            .keywords
                            .push(Self::extract_text(reader, buf, b"keyword")?);
                    }
                    b"language" => {
                        citation.language = Some(Self::extract_text(reader, buf, b"language")?);
                    }
                    b"publisher" => {
                        citation.publisher = Some(Self::extract_text(reader, buf, b"publisher")?);
                    }
                    b"isbn" => {
                        let issns = Self::extract_text(reader, buf, b"isbn")?;
                        citation.issn.extend(split_issns(&issns));
                    }
                    _ => (),
                },
                Ok(Event::End(ref e)) if e.name() == QName(b"record") => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(CitationError::from(e)),
                _ => (),
            }
            buf.clear();
        }

        Ok(citation)
    }
}

impl CitationParser for EndNoteXmlParser {
    fn parse(&self, input: &str) -> Result<Vec<Citation>> {
        if input.trim().is_empty() {
            return Err(CitationError::InvalidFormat("Empty input".into()));
        }

        let mut reader = Reader::from_str(input);
        reader.config_mut().trim_text(true);

        let mut citations = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name() == QName(b"record") => {
                    citations.push(self.parse_record(&mut reader, &mut buf)?); // Changed to use self.parse_record
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(CitationError::from(e)),
                _ => (),
            }
            buf.clear();
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
    fn test_parse_basic_citation() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
        <xml><records><record>
        <ref-type name="Journal Article"/>
        <titles>
            <title>Test Article Title</title>
            <secondary-title>Test Journal</secondary-title>
        </titles>
        <contributors>
            <authors>
                <author>Smith, John</author>
                <author>Doe, Jane</author>
            </authors>
        </contributors>
        <volume>10</volume>
        <number>2</number>
        <pages>100-110</pages>
        <year>2023</year>
        <electronic-resource-num>10.1000/test</electronic-resource-num>
        <abstract>This is a test abstract.</abstract>
        <keywords><keyword>Test</keyword><keyword>XML</keyword></keywords>
        </record></records></xml>"#;

        let parser = EndNoteXmlParser::new();
        let result = parser.parse(input).unwrap();

        assert_eq!(result.len(), 1);
        let citation = &result[0];
        assert_eq!(citation.citation_type[0], "Journal Article");
        assert_eq!(citation.title, "Test Article Title");
        assert_eq!(citation.authors.len(), 2);
        assert_eq!(citation.authors[0].family_name, "Smith");
        assert_eq!(citation.authors[1].family_name, "Doe");
        assert_eq!(citation.journal, Some("Test Journal".to_string()));
        assert_eq!(citation.year, Some(2023));
        assert_eq!(citation.volume, Some("10".to_string()));
        assert_eq!(citation.issue, Some("2".to_string()));
        assert_eq!(citation.pages, Some("100-110".to_string()));
        assert_eq!(citation.doi, Some("10.1000/test".to_string()));
        assert_eq!(
            citation.abstract_text,
            Some("This is a test abstract.".to_string())
        );
        assert_eq!(citation.keywords, vec!["Test", "XML"]);
    }

    #[test]
    fn test_parse_multiple_citations() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
        <xml><records>
        <record>
            <titles><title>First Article</title></titles>
            <contributors><authors><author>Smith, J</author></authors></contributors>
        </record>
        <record>
            <titles><title>Second Article</title></titles>
            <contributors><authors><author>Doe, J</author></authors></contributors>
        </record>
        </records></xml>"#;

        let parser = EndNoteXmlParser::new();
        let result = parser.parse(input).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "First Article");
        assert_eq!(result[1].title, "Second Article");
    }

    #[test]
    fn test_parse_malformed_xml() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
        <xml><records><record>
        <title>Incomplete Record"#;

        let parser = EndNoteXmlParser::new();
        assert!(parser.parse(input).is_err());
    }

    #[test]
    fn test_parse_multiple_issns() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
        <xml><records><record>
        <titles><title>Test Article</title></titles>
        <isbn>1234-5678 (Print)\r\n5678-1234 (Electronic)\r0047-1852 (Print)\n0047-1852</isbn>
        </record></records></xml>"#;

        let parser = EndNoteXmlParser::new();
        let result = parser.parse(input).unwrap();

        assert_eq!(
            result[0].issn,
            vec![
                "1234-5678 (Print)",
                "5678-1234 (Electronic)",
                "0047-1852 (Print)",
                "0047-1852"
            ]
        );
    }
}
