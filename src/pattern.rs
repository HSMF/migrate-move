#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Order {
    IndexOnly,
    IndexFirst,
    NameFirst,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pattern<'a> {
    pub parts: Vec<&'a str>,
    pub(crate) order: Order,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ParseError {
    #[error("invalid pattern")]
    InvalidPattern,
    #[error("unfinished substitute")]
    UnfinishedSubstitute,
    #[error("too many placeholders")]
    TooManyPlaceholders,
}

impl<'a> Pattern<'a> {
    pub fn parse(s: &'a str) -> Result<Self, ParseError> {
        let mut iter = s.char_indices();
        let mut parts = vec![];
        let mut id_index = None;
        let mut name_index = None;

        let mut start = 0;
        while let Some((i, next)) = iter.next() {
            if let '%' = next {
                let (j, after) = iter.next().ok_or(ParseError::UnfinishedSubstitute)?;

                match after {
                    'd' => {
                        if id_index.is_some() {
                            return Err(ParseError::TooManyPlaceholders);
                        }
                        id_index = Some(parts.len())
                    }
                    's' => {
                        if name_index.is_some() {
                            return Err(ParseError::TooManyPlaceholders);
                        }
                        name_index = Some(parts.len())
                    }
                    _ => return Err(ParseError::InvalidPattern),
                }
                parts.push(&s[start..i]);
                start = j + after.len_utf8();
            }
        }

        if start != s.len() {
            parts.push(&s[start..]);
        }

        let id_index = id_index.ok_or(ParseError::InvalidPattern)?;
        let order = match (id_index, name_index) {
            (_, None) => Order::IndexOnly,
            (i, Some(j)) if i < j => Order::IndexFirst,
            _ => Order::NameFirst,
        };

        Ok(Pattern { parts, order })
    }
}

#[cfg(test)]
mod tests {
    use super::{Order, Pattern};

    #[test]
    fn parse_normal() {
        let pat = "%d_%s.up.sql";
        let pat = Pattern::parse(pat);

        assert_eq!(
            pat,
            Ok(Pattern {
                parts: vec!["", "_", ".up.sql"],
                order: Order::IndexFirst
            })
        )
    }
}
