/// An [Iterator] which inclusively splits text on blank new lines.
///
/// [Iterator::next] returns chunks of consecutive non-blank lines,
/// along with their starting line number.
pub(crate) struct BlankLineSplit<'a> {
    line_number: usize,
    text: &'a str,
    line_break: &'a str,
}

impl<'a> BlankLineSplit<'a> {
    pub(crate) fn new(text: &'a str, line_break: &'a str) -> Self {
        Self {
            line_number: 1,
            text,
            line_break,
        }
    }
}

impl<'a> Iterator for BlankLineSplit<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let mut i = 0;
        let mut lines = 0;
        let mut split = self.text.split_inclusive(self.line_break);
        for line in split.by_ref() {
            lines += 1;
            i += line.len();
            if line == self.line_break {
                break;
            }
        }
        while split.next().is_some_and(|line| line == self.line_break) {
            lines += 1;
            i += self.line_break.len();
        }
        if i == 0 {
            None
        } else {
            let (part, rest) = self.text.split_at(i);
            let line_number = self.line_number;
            self.text = rest;
            self.line_number += lines;
            Some((line_number, part))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use pretty_assertions::assert_eq;
    use rstest::*;

    #[rstest]
    #[case("", &[])]
    #[case("\n", &[(1, "\n")])]
    #[case("\n\n", &[(1, "\n\n")])]
    #[case("\n\n\n", &[(1, "\n\n\n")])]
    #[case("\n\n\n\n", &[(1, "\n\n\n\n")])]
    #[case("one", &[(1, "one")])]
    #[case("\none", &[(1, "\n"), (2, "one")])]
    #[case("\n\none", &[(1, "\n\n"), (3, "one")])]
    #[case("one\n", &[(1, "one\n")])]
    #[case("one\ntwo\nthree\n", &[(1, "one\ntwo\nthree\n")])]
    #[case("one\ntwo\nthree\n\n\n", &[(1, "one\ntwo\nthree\n\n\n")])]
    #[case("one\ntwo\nthree\n\napple\nbat\ncat\n", &[(1, "one\ntwo\nthree\n\n"), (5, "apple\nbat\ncat\n")])]
    #[case("one\ntwo\nthree\n\n\napple\nbat\ncat\n", &[(1, "one\ntwo\nthree\n\n\n"), (6, "apple\nbat\ncat\n")])]
    #[case("one\ntwo\nthree\n\n\n\napple\nbat\ncat\n", &[(1, "one\ntwo\nthree\n\n\n\n"), (7, "apple\nbat\ncat\n")])]
    #[case("one\ntwo\nthree\n\n\n\napple\nbat\ncat\n\n\n", &[(1, "one\ntwo\nthree\n\n\n\n"), (7, "apple\nbat\ncat\n\n\n")])]
    #[case("\n\none\ntwo\nthree\n\n\n\napple\nbat\ncat\n", &[(1, "\n\n"), (3, "one\ntwo\nthree\n\n\n\n"), (9, "apple\nbat\ncat\n")])]
    fn test_blank_line_split_empty(#[case] text: &str, #[case] expected: &[(usize, &str)]) {
        let actual = BlankLineSplit::new(text, "\n").collect_vec();
        assert_eq!(&actual, expected)
    }
}
