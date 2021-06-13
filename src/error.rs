use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub enum Kind {
    NotCommand,
    NotParameter,
    CantExecuteParameter,
    ValueParseFailed,
    ParameterValueMissed,
    ParserError,
    CantExecuteCommand,
}

#[derive(Debug)]
pub struct Error {
    pub(super) kind: Kind,
    pub(super) details: String
}

impl Error {
    pub fn kind(&self) -> Kind { self.kind.clone() }
    pub fn description(&self) -> &str { self.details.as_str() }
}

impl std::error::Error for self::Error {}

impl std::fmt::Display for self::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error {}: {}", self.kind, self.details))
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Kind {
    pub fn as_str(&self) -> &str {
        match self {
            Kind::NotCommand => "Not a command",
            Kind::NotParameter => "Not valid parameter",
            Kind::CantExecuteParameter => "Can't execute parameter, please specify a command",
            Kind::ValueParseFailed => "Value parse failed",
            Kind::ParameterValueMissed => "Missed parameter value",
            Kind::ParserError => "Parser error. Make an issue",
            Kind::CantExecuteCommand => "Can't execute command, because subcommand not provided"
        }
    }
}