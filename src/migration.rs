use std::{
    fmt::Write,
    num::{IntErrorKind, ParseIntError},
};

use crate::pattern::{Order, Pattern};

#[derive(Debug)]
pub struct Migration {
    pub name: String,
    pub id: usize,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("filename doesn't match")]
    NotMatching,

    #[error("id is overflowing")]
    IdOverflowing,
}

impl Migration {
    /// not correct but i do not care
    pub fn from_pattern(pattern: &Pattern<'_>, s: &str) -> Result<Self, ParseError> {
        let s = &s[s.find(pattern.parts[0]).ok_or(ParseError::NotMatching)?..];

        match pattern.order {
            Order::IndexOnly => {
                let s = s
                    .strip_suffix(pattern.parts[1])
                    .ok_or(ParseError::NotMatching)?;

                let id = s.parse().map_err(|err: ParseIntError| match err.kind() {
                    IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                        ParseError::IdOverflowing
                    }
                    _ => ParseError::NotMatching,
                })?;

                Ok(Migration {
                    name: "".to_string(),
                    id,
                })
            }

            Order::IndexFirst => {
                let (id, s) = s
                    .split_once(pattern.parts[1])
                    .ok_or(ParseError::NotMatching)?;

                let id = id.parse().map_err(|err: ParseIntError| match err.kind() {
                    IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                        ParseError::IdOverflowing
                    }
                    _ => ParseError::NotMatching,
                })?;

                let name = s
                    .strip_suffix(pattern.parts[2])
                    .ok_or(ParseError::NotMatching)?
                    .to_string();

                Ok(Migration { name, id })
            }

            Order::NameFirst => {
                let (name, s) = s
                    .split_once(pattern.parts[1])
                    .ok_or(ParseError::NotMatching)?;
                let name = name.to_string();

                let id = s
                    .strip_suffix(pattern.parts[1])
                    .ok_or(ParseError::NotMatching)?;

                let id = id.parse().map_err(|err: ParseIntError| match err.kind() {
                    IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                        ParseError::IdOverflowing
                    }
                    _ => ParseError::NotMatching,
                })?;

                Ok(Migration { name, id })
            }
        }
    }

    pub fn to_string(&self, pattern: &Pattern<'_>) -> String {
        let mut out = String::new();

        match pattern.order {
            Order::IndexOnly => {
                out.push_str(pattern.parts[0]);
                write!(&mut out, "{}", self.id).expect("not supposed to error");
                out.push_str(pattern.parts[1]);
            }
            Order::IndexFirst => {
                out.push_str(pattern.parts[0]);
                write!(&mut out, "{}", self.id).expect("not supposed to error");
                out.push_str(pattern.parts[1]);
                write!(&mut out, "{}", self.name).expect("not supposed to error");
                out.push_str(pattern.parts[2]);
            }
            Order::NameFirst => {
                out.push_str(pattern.parts[0]);
                write!(&mut out, "{}", self.name).expect("not supposed to error");
                out.push_str(pattern.parts[1]);
                write!(&mut out, "{}", self.id).expect("not supposed to error");
                out.push_str(pattern.parts[2]);
            }
        }

        out
    }
}
