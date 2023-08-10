use thiserror::Error as ThisError;

#[derive(ThisError, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("Not a command")]
    NotCommand,
    #[error("Not valid parameter")]
    NotParameter,
    #[error("Can't execute parameter, please specify a command")]
    CantExecuteParameter,
    #[error("Value parse failed")]
    ValueParseFailed,
    #[error("Missed parameter value")]
    ParameterValueMissed,
    #[error("Parser error. Make an issue")]
    ParserError,
    #[error("Can't execute command, because subcommand not provided")]
    CantExecuteCommand,
}
