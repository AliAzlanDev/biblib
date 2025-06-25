use std::ops::Add;

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
pub(crate) struct WholeLinesIter<'a, I: Iterator<Item = &'a str>> {
    lines: I,
    current: Option<&'a str>,
}

impl<'a, I: Iterator<Item = &'a str>> WholeLinesIter<'a, I> {
    /// Create a new [WholeLinesIter].
    pub(crate) fn new(mut lines: I) -> Self {
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
    use itertools::Itertools;
    use rstest::*;
    use super::*;

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
            &actual.iter().map(|s| s.as_str()).collect_vec(),
            expected
        )
    }
}