//! EndNote XML parsing implementation.
//!
//! This module provides the core parsing logic for EndNote XML format.

use crate::{Author, Citation, CitationFormat};
use crate::error::{ParseError, ValueError};
use quick_xml::Reader;
use quick_xml::events::Event;
use quick_xml::name::QName;
use std::io::BufRead;

/// Convert buffer position to approximate line number
fn buffer_position_to_line_number(content: &str, pos: usize) -> usize {
    if pos >= content.len() {
        return content.lines().count();
    }
    content[..pos].lines().count()
}

/// Enhanced extract_text function that tracks line numbers for better error reporting
fn extract_text_with_position<B: BufRead>(
    reader: &mut Reader<B>,
    buf: &mut Vec<u8>,
    closing_tag: &[u8],
    content: &str,
    _start_pos: usize,
) -> Result<String, ParseError> {
    let mut text = String::new();
    let closing_tag_str = String::from_utf8_lossy(closing_tag);

    loop {
        let current_pos = reader.buffer_position() as usize;
        match reader.read_event_into(buf) {
            Ok(Event::Text(e)) => {
                text.push_str(&e.unescape().map_err(|e| {
                    let line_num = buffer_position_to_line_number(content, current_pos);
                    ParseError::at_line(
                        line_num,
                        CitationFormat::EndNoteXml,
                        ValueError::Syntax(format!("Invalid XML text content: {}", e))
                    )
                })?);
            }
            Ok(Event::End(e)) if e.name() == QName(closing_tag) => break,
            Ok(Event::Eof) => {
                let line_num = buffer_position_to_line_number(content, current_pos);
                return Err(ParseError::at_line(
                    line_num,
                    CitationFormat::EndNoteXml,
                    ValueError::Syntax(format!(
                        "Unexpected EOF while looking for closing tag '{}'",
                        closing_tag_str
                    ))
                ));
            }
            Err(e) => {
                let line_num = buffer_position_to_line_number(content, current_pos);
                return Err(ParseError::at_line(
                    line_num,
                    CitationFormat::EndNoteXml,
                    ValueError::Syntax(format!("XML parsing error: {}", e))
                ));
            }
            _ => continue,
        }
        buf.clear();
    }

    Ok(text.trim().to_string())
}

/// Parse EndNote XML content into citations.
///
/// This function parses EndNote XML format and returns a vector of citations.
/// It uses quick_xml for robust XML parsing and converts to our Citation structure.
///
/// # Arguments
///
/// * `content` - The EndNote XML content as a string
///
/// # Returns
///
/// A Result containing either a vector of citations or a parsing error.
///
/// ```
pub(crate) fn parse_endnote_xml(content: &str) -> Result<Vec<Citation>, ParseError> {
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut citations = Vec::new();
    let mut buf = Vec::new();

    loop {
        let pos = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == QName(b"record") => {
                citations.push(parse_record(&mut reader, &mut buf, content, pos)?);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ParseError::from(e)),
            _ => (),
        }
        buf.clear();
    }

    // Return empty vector instead of error for empty but valid XML
    Ok(citations)
}

/// Extracts text content from XML events until the closing tag is found
fn extract_text<B: BufRead>(
    reader: &mut Reader<B>,
    buf: &mut Vec<u8>,
    closing_tag: &[u8],
) -> Result<String, ParseError> {
    let mut text = String::new();
    let closing_tag_str = String::from_utf8_lossy(closing_tag);

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Text(e)) => {
                text.push_str(&e.unescape().map_err(|e| {
                    ParseError::without_position(
                        CitationFormat::EndNoteXml,
                        ValueError::Syntax(format!("Invalid XML text content: {}", e))
                    )
                })?);
            }
            Ok(Event::End(e)) if e.name() == QName(closing_tag) => break,
            Ok(Event::Eof) => {
                return Err(ParseError::without_position(
                    CitationFormat::EndNoteXml,
                    ValueError::Syntax(format!(
                        "Unexpected EOF while looking for closing tag '{}'",
                        closing_tag_str
                    ))
                ));
            }
            Err(e) => return Err(ParseError::from(e)),
            _ => continue,
        }
        buf.clear();
    }

    Ok(text.trim().to_string())
}

/// Extracts date components (year, month, day) from a year element
fn extract_date_from_year_element<B: BufRead>(
    reader: &mut Reader<B>,
    e: &quick_xml::events::BytesStart,
    content: &str,
) -> Result<(Option<i32>, Option<u8>, Option<u8>), ParseError> {
    let mut year_val = None;
    let mut month_val = None;
    let mut day_val = None;

    // First, extract attributes if present
    for attr in e.attributes() {
        let attr = attr.map_err(|e| ParseError::without_position(
            CitationFormat::EndNoteXml,
            ValueError::Syntax(format!("Invalid attribute: {}", e))
        ))?;
        match attr.key.as_ref() {
            b"year" => {
                if let Ok(year_str) = std::str::from_utf8(&attr.value) {
                    year_val = year_str.parse::<i32>().ok();
                }
            }
            b"month" => {
                if let Ok(month_str) = std::str::from_utf8(&attr.value) {
                    month_val = month_str
                        .parse::<u8>()
                        .ok()
                        .filter(|&m| (1..=12).contains(&m));
                }
            }
            b"day" => {
                if let Ok(day_str) = std::str::from_utf8(&attr.value) {
                    day_val = day_str
                        .parse::<u8>()
                        .ok()
                        .filter(|&d| (1..=31).contains(&d));
                }
            }
            _ => {}
        }
    }

    // If no year attribute, try to get year from text content
    if year_val.is_none() {
        let mut local_buf = Vec::new();
        let start_pos = reader.buffer_position() as usize;
        if let Ok(year) =
            extract_text_with_position(reader, &mut local_buf, b"year", content, start_pos)?
                .parse::<i32>()
        {
            year_val = Some(year);
        }
    } else {
        // Still need to consume the text content
        let mut local_buf = Vec::new();
        let start_pos = reader.buffer_position() as usize;
        let _ = extract_text_with_position(reader, &mut local_buf, b"year", content, start_pos)?;
    }

    Ok((year_val, month_val, day_val))
}

/// Parse a single record element into a Citation
fn parse_record<B: BufRead>(
    reader: &mut Reader<B>,
    buf: &mut Vec<u8>,
    content: &str,
    start_pos: usize,
) -> Result<Citation, ParseError> {
    let mut citation = Citation::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"ref-type" => {
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| ParseError::without_position(
                            CitationFormat::EndNoteXml,
                            ValueError::Syntax(format!("Invalid attribute: {}", e))
                        ))?;
                        if attr.key.as_ref() == b"name" {
                            citation.citation_type.push(
                                attr.unescape_value()
                                    .map_err(|e| ParseError::without_position(
                                        CitationFormat::EndNoteXml,
                                        ValueError::Syntax(format!("Invalid attribute value: {}", e))
                                    ))?
                                    .into_owned(),
                            );
                        }
                    }
                }
                b"title" => {
                    citation.title = extract_text(reader, buf, b"title")?;
                }
                b"author" => {
                    let author_str = extract_text(reader, buf, b"author")?;
                    let (family, given) = crate::utils::parse_author_name(&author_str);
                    citation.authors.push(Author {
                        family_name: family,
                        given_name: given,
                        affiliation: None,
                    });
                }
                b"secondary-title" => {
                    let sec_title = extract_text(reader, buf, b"secondary-title")?;
                    // If no primary title, use secondary-title as title
                    if citation.title.is_empty() {
                        citation.title = sec_title;
                    } else {
                        citation.journal = Some(sec_title);
                    }
                }
                b"alt-title" => {
                    let alt_title = extract_text(reader, buf, b"alt-title")?;
                    // If no primary title or journal is set, use alt-title as title
                    if citation.title.is_empty() && citation.journal.is_none() {
                        citation.title = alt_title;
                    } else if citation.journal.is_none() {
                        citation.journal = Some(alt_title);
                    } else {
                        citation.journal_abbr = Some(alt_title);
                    }
                }
                b"custom2" => {
                    let text = extract_text(reader, buf, b"custom2")?;
                    // Check for PMC ID patterns
                    if text.to_lowercase().contains("pmc") || text.starts_with("PMC") {
                        citation.pmc_id = Some(text);
                    }
                }
                b"volume" => {
                    citation.volume = Some(extract_text(reader, buf, b"volume")?);
                }
                b"number" => {
                    citation.issue = Some(extract_text(reader, buf, b"number")?);
                }
                b"pages" => {
                    let pages = extract_text(reader, buf, b"pages")?;
                    citation.pages = Some(crate::utils::format_page_numbers(&pages));
                }
                b"electronic-resource-num" => {
                    let doi = extract_text(reader, buf, b"electronic-resource-num")?;
                    citation.doi = crate::utils::format_doi(&doi);
                }
                b"url" => {
                    let url = extract_text(reader, buf, b"url")?;
                    if citation.doi.is_none() && url.contains("doi.org") {
                        citation.doi = crate::utils::format_doi(&url);
                    }
                    citation.urls.push(url);
                }
                b"year" => {
                    let (year_val, month_val, day_val) =
                        extract_date_from_year_element(reader, e, content)?;
                    citation.date = crate::utils::parse_endnote_date(year_val, month_val, day_val);
                    // For backward compatibility, also set the deprecated year field
                    #[allow(deprecated)]
                    {
                        citation.year = citation.date.as_ref().map(|d| d.year);
                    }
                }
                b"dates" => {
                    // Handle the dates element - we'll look for year sub-element
                    // This is a more complex structure but we'll process it
                    loop {
                        match reader.read_event_into(buf) {
                            Ok(Event::Start(ref inner_e)) if inner_e.name() == QName(b"year") => {
                                // Parse year element within dates
                                let (year_val, month_val, day_val) =
                                    extract_date_from_year_element(reader, inner_e, content)?;
                                citation.date =
                                    crate::utils::parse_endnote_date(year_val, month_val, day_val);
                                // For backward compatibility, also set the deprecated year field
                                #[allow(deprecated)]
                                {
                                    citation.year = citation.date.as_ref().map(|d| d.year);
                                }
                            }
                            Ok(Event::End(ref inner_e)) if inner_e.name() == QName(b"dates") => {
                                break;
                            }
                            Ok(Event::Eof) => break,
                            Err(e) => return Err(ParseError::from(e)),
                            _ => continue,
                        }
                        buf.clear();
                    }
                }
                b"abstract" => {
                    citation.abstract_text = Some(extract_text(reader, buf, b"abstract")?);
                }
                b"keyword" => {
                    citation
                        .keywords
                        .push(extract_text(reader, buf, b"keyword")?);
                }
                b"language" => {
                    citation.language = Some(extract_text(reader, buf, b"language")?);
                }
                b"publisher" => {
                    citation.publisher = Some(extract_text(reader, buf, b"publisher")?);
                }
                b"isbn" => {
                    let issns = extract_text(reader, buf, b"isbn")?;
                    citation.issn.extend(crate::utils::split_issns(&issns));
                }
                _ => (),
            },
            Ok(Event::End(ref e)) if e.name() == QName(b"record") => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(ParseError::from(e)),
            _ => (),
        }
        buf.clear();
    }

    // Validate that we have at least a title or author
    if citation.title.is_empty() && citation.authors.is_empty() {
        let line_num = buffer_position_to_line_number(content, start_pos);
        return Err(ParseError::at_line(
            line_num,
            CitationFormat::EndNoteXml,
            ValueError::MissingValue {
                field: "title or author",
                key: "title/author",
            }
        ));
    }

    Ok(citation)
}
