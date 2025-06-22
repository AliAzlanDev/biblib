use crate::pubmed::author::{resolve_authors, ConsecutiveTag};
use crate::pubmed::structure::RawPubmedData;
use crate::pubmed::tags::PubmedTag;
use either::{Either, Left, Right};
use itertools::Itertools;
use std::collections::HashMap;
use std::ops::Add;

/// Parse the content of a PubMed formatted .nbib file, returning its key-value pairs
/// in a [HashMap] (with the order of duplicate values preserved in the [Vec] values)
/// alongside any unparsable lines.
pub fn pubmed_parse<S: AsRef<str>>(nbib_text: S) -> RawPubmedData {
    let text = nbib_text.as_ref();
    let (mut ignored_lines, pairs): (Vec<_>, Vec<_>) =
        WholeLinesIter::new(text.split('\n')).partition_map(parse_complete_entry);
    let (data, others) = separate_stateless_entries(pairs);
    let (authors, leading_affiliations) = resolve_authors(others);
    ignored_lines.extend(
        leading_affiliations
            .into_iter()
            .map(|s| format!("AD - {s}")),
    );
    RawPubmedData {
        data,
        authors,
        ignored_lines,
    }
}

/// Collect the data: tags which can be parsed statelessly are stored in a [HashMap],
/// with duplicates kept in a [Vec] with order preserved, while other tags that require
/// context to parse are stored in a vec with order preserved.
#[allow(clippy::type_complexity)]
fn separate_stateless_entries<V>(
    v: Vec<(PubmedTag, V)>,
) -> (HashMap<PubmedTag, Vec<V>>, Vec<(ConsecutiveTag, V)>) {
    let mut map = HashMap::with_capacity(v.len());
    let mut other = Vec::with_capacity(v.len());
    for (k, v) in v {
        if let Some(tag) = ConsecutiveTag::from_tag(k) {
            other.push((tag, v))
        } else {
            let bucket = map.entry(k).or_insert_with(Vec::new);
            bucket.push(v);
        }
    }
    (map, other)
}

/// Parse the string as a key-value pair from a PubMed formatted .nbib file.
fn parse_complete_entry(line: String) -> Either<String, (PubmedTag, String)> {
    split_on_dash(&line)
        .and_then(|(k, v)| match_pubmed_key(k, v))
        .map(|(k, v)| Right((k, v.to_string())))
        .unwrap_or_else(|| Left(line))
}

/// Match `key` with a known [PubmedTag].
fn match_pubmed_key<S: AsRef<str>, V>(key: S, value: V) -> Option<(PubmedTag, V)> {
    PubmedTag::from_tag(key.as_ref()).map(|tag| (tag, value))
}

/// Split on the first `-` character and remove the whitespace surrounding the removed `-`.
fn split_on_dash(line: &str) -> Option<(&str, &str)> {
    line.split_once('-')
        .map(|(l, r)| (l.trim_end(), r.trim_start()))
}

/// An [Iterator] which yields lines containing whole values from a PubMed .nbib formatted string.
///
/// The PubMed .nbib file format consists of key-value pairs e.g.
///
/// ```plain
/// PMC - PMC11227906
/// ```
///
/// However, values may be split on whitespace and occupy multiple lines, e.g.
///
/// ```plain
/// TI  - Fantastic yeasts and where to find them: the hidden diversity of dimorphic fungal
///       pathogens.
/// ```
///
/// [WholeLinesIter] joins multi-line key-value pairs into a single string, so:
///
/// ```plain
/// TI  - Fantastic yeasts and where to find them: the hidden diversity of dimorphic fungal pathogens.
/// ```
struct WholeLinesIter<'a, I: Iterator<Item = &'a str>> {
    lines: I,
    current: Option<&'a str>,
}

impl<'a, I: Iterator<Item = &'a str>> WholeLinesIter<'a, I> {
    /// Create a new [WholeLinesIter].
    fn new(mut lines: I) -> Self {
        Self {
            current: lines.next(),
            lines,
        }
    }

    /// Consume items from `self.lines` until the next key-value pair is reached.
    /// Sets `self.current` to be the first line of the next key-value pair,
    /// then return the consumed previous key-value pair.
    fn consume_complete_value(&mut self, first_line: &'a str) -> String {
        let mut value = vec![first_line];
        loop {
            if let Some(line) = self.lines.next() {
                if line.starts_with(' ') {
                    // continuation of previous value
                    value.push(line.trim_start());
                } else {
                    // start of next key-value pair
                    self.current = Some(line);
                    break;
                }
            } else {
                // end of .nbib file
                self.current = None;
                break;
            }
        }
        join_lines(value)
    }
}

/// Join strings on space, except for hyphen-terminated or blank items which are joined without a space.
fn join_lines(v: Vec<&str>) -> String {
    v.into_iter().fold(String::new(), |acc, e| {
        if acc.ends_with('-') || acc.ends_with(' ') || acc.is_empty() {
            acc
        } else {
            acc.add(" ")
        }
        .add(e)
    })
}

impl<'a, I: Iterator<Item = &'a str>> Iterator for WholeLinesIter<'a, I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|x| self.consume_complete_value(x))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("", &[""])]
    #[case(r#"PMID- 123456
FOO - bar
BOB - Alice"#, &["PMID- 123456", "FOO - bar", "BOB - Alice"])]
    #[
    case(r#"PMID- 123456
FOO - bar
LONG- I am a very long line containing so
      much text that there is a line break"#,
&["PMID- 123456", "FOO - bar", "LONG- I am a very long line containing so much text that there is a line break"])]
    #[
    case(r#"PMID- 123456
LONG- Self-
      assembled structures are important"#,
&["PMID- 123456", "LONG- Self-assembled structures are important"])]
    #[
    case(r#"PMID- 123456
FOO - bar
LONG- I am a very long line containing so
      much text that there is a line break
LAST- line
"#,
&["PMID- 123456", "FOO - bar", "LONG- I am a very long line containing so much text that there is a line break", "LAST- line", ""])]
    fn test_continued_lines_iterator(#[case] text: &str, #[case] expected: &[&str]) {
        let actual: Vec<_> = WholeLinesIter::new(text.split('\n')).collect();
        assert_eq!(
            &actual.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            expected
        )
    }

    #[rstest]
    #[case("", Left(""))]
    #[case("DNE - tag does not exist", Left("DNE - tag does not exist"))]
    #[case("AU - Albert Einstein", Right((PubmedTag::Author, "Albert Einstein")))]
    #[case("AU- Albert Einstein", Right((PubmedTag::Author, "Albert Einstein")))]
    #[case("AU -Albert Einstein", Right((PubmedTag::Author, "Albert Einstein")))]
    #[case("AU  - Albert Einstein", Right((PubmedTag::Author, "Albert Einstein")))]
    fn test_parse_complete_entry(
        #[case] line: &str,
        #[case] expected: Either<&str, (PubmedTag, &str)>,
    ) {
        let actual = parse_complete_entry(line.to_string());
        assert_eq!(
            actual
                .as_ref()
                .map_either(|s| s.as_str(), |(t, s)| (t.clone(), s.as_str())),
            expected
        )
    }
}
