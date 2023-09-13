use std::fmt::Display;

use thiserror::Error as ThisError;

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

#[derive(ThisError, Debug, Clone, PartialEq)]
pub enum Error<'a> {
    #[error("Not a command: {0}")]
    NotCommand(Span<'a>),
    #[error("Not valid parameter: {0}")]
    NotParameter(Span<'a>),
    #[error("Command expected, got: {0}")]
    CommandExpected(Span<'a>),
    #[error("Missed parameter value")]
    ParameterValueMissed,
    #[error("Parser error. Make an issue")]
    ParserFault,
    #[error("No handler for command: {0}")]
    NoHandler(&'a str),
    #[error("Not a value")]
    NotValue(Span<'a>),
    #[error(
        "Not a boolean value. \
    Use \"1\", \"true\", \"yes\", \"on\" for true, \
    and \"0\", \"false\", \"no\", \"off\" for false"
    )]
    ParseBool(Span<'a>),
    #[error("Parse int error: {1}")]
    ParseInt(Span<'a>, std::num::ParseIntError),
    #[error("Parse float error: {1}")]
    ParseFloat(Span<'a>, std::num::ParseFloatError),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Span<'a> {
    pub source: &'a str,
    pub begin: usize,
    pub end: usize,
}

impl<'a> Span<'a> {
    pub fn arg(&'a self) -> &'a str {
        &self.source[self.begin..self.end]
    }
}

impl<'a> Display for Span<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.arg().fmt(f)
    }
}
