use crate::traits::Config;

use super::command::*;
use super::context::*;
use super::error::*;
use super::parameter::*;

use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    rc::Rc,
    str::FromStr,
};

///  `Cli` is a central unit that contains all possible commands, arguments and handlers.
/// To create instance using build pattern. Generic parameter using for return values from handlers.
///
/// # Example
///
/// ```rust
/// use clean_cli::*;
///
/// let cli = <Cli<DefaultConfig<bool>>>::builder()
///      .command(CommandBuilder::with_name("cmd")
///          .handler(|ctx| {
///              // some logic
///              return true;
///          })
///      )
///      .build();
///
/// assert!(cli.exec_line("cmd").unwrap());
/// ```
#[derive(Debug)]
pub struct Cli<T: Config> {
    root: (String, Rc<Command<T>>),
    printer: T::Printer,
    need_print_error: bool,
    need_print_help: bool,
}

impl<T: Config> Cli<T> {
    /// Create builder
    pub fn builder() -> CliBuilder<T> {
        CliBuilder {
            commands: Default::default(),
            printer: None,
            need_print_error: Default::default(),
            need_print_help: Default::default(),
        }
    }

    /// Execute _line_
    pub fn exec_line(&self, line: &str) -> Result<T::Result, Error> {
        enum ParseState {
            ReadFirst,
            ReadNext,
            ParametersReaded { params: VecDeque<Rc<Parameter>> },
        }

        let mut ctx = Context::<T> {
            units: vec![ContextUnit {
                command: (self.root.0.as_str(), self.root.1.clone()),
                parameters: Default::default(),
                value: None,
            }],
            printer: &self.printer,
        };
        let mut state = ParseState::ReadFirst;
        let mut pos = 1;
        let args = split_line(line);

        for (arg, span) in args {
            match state {
                ParseState::ReadFirst => {
                    if arg.starts_with("--") || arg.starts_with('-') {
                        return self.make_error(Error::CantExecuteParameter, arg.to_string());
                    } else if let Some(cmd) = self.commands().get(arg) {
                        ctx.units.push(ContextUnit {
                            command: (arg, cmd.clone()),
                            parameters: Default::default(),
                            value: None,
                        });
                        state = ParseState::ReadNext;
                    } else {
                        return self.make_error(Error::NotCommand, arg.to_string());
                    }
                }

                ParseState::ReadNext => {
                    let last_unit = &mut ctx.units[pos];
                    let cmd = last_unit.command.1.clone();
                    let mut new_state: Option<ParseState> = None;

                    if let Some(arg) = arg.strip_prefix("--") {
                        if let Some(p) = cmd.parameters.get(arg) {
                            if let ArgType::Bool = p.value_type {
                                if let Some(p) = last_unit.command.1.parameters.get(arg) {
                                    last_unit
                                        .parameters
                                        .insert(p.name.clone(), (p.clone(), ArgValue::Bool(true)));
                                }
                            } else {
                                let mut params = VecDeque::with_capacity(1);
                                params.push_back(p.clone());
                                new_state = Some(ParseState::ParametersReaded { params });
                            }
                        } else {
                            return self.make_error(Error::NotParameter, arg.to_string());
                        }
                    } else if let Some(arg) = arg.strip_prefix('-') {
                        let mut params = VecDeque::with_capacity(arg.len());
                        for a in arg.chars() {
                            let s = a.to_string();
                            if let Some(p) = cmd.parameters.get(&s) {
                                if let ArgType::Bool = p.value_type {
                                    if let Some(p) = last_unit.command.1.parameters.get(&s) {
                                        last_unit.parameters.insert(
                                            p.name.clone(),
                                            (p.clone(), ArgValue::Bool(true)),
                                        );
                                    }
                                } else {
                                    params.push_back(p.clone());
                                }
                            } else {
                                return self.make_error(Error::NotParameter, arg.to_string());
                            }
                        }

                        if !params.is_empty() {
                            new_state = Some(ParseState::ParametersReaded { params });
                        }
                    } else if let Some(sub) = cmd.subcommands.get(arg) {
                        ctx.units.push(ContextUnit {
                            command: (arg, sub.clone()),
                            parameters: Default::default(),
                            value: None,
                        });
                        pos += 1;
                        new_state = Some(ParseState::ReadNext);
                    } else if let Some(v) = cmd.value.as_ref() {
                        match parse_arg(v.clone(), arg) {
                            Ok(value) => {
                                last_unit.value = Some(value);
                            }
                            Err(details) => {
                                return self.make_error(Error::ValueParseFailed, details)
                            }
                        };
                    } else {
                        return self.make_error(Error::NotCommand, arg.to_string());
                    }

                    if let Some(s) = new_state {
                        state = s;
                    }
                }

                ParseState::ParametersReaded { mut params } => {
                    let last_unit = &mut ctx.units[pos];

                    let param = params.pop_front().unwrap();
                    match parse_arg(param.value_type.clone(), arg) {
                        Ok(value) => {
                            last_unit
                                .parameters
                                .insert(param.name.clone(), (param.clone(), value));
                            if params.is_empty() {
                                state = ParseState::ReadNext;
                            } else {
                                state = ParseState::ParametersReaded { params };
                            }
                        }
                        Err(details) => {
                            return self.make_error(Error::ValueParseFailed, details);
                        }
                    };
                }
            }
        }

        if let ParseState::ParametersReaded { mut params } = state {
            if params.len() > 1 {
                return self.make_error(
                    Error::ParserError,
                    format!("Wrong params value: {}", params.len()),
                );
            }
            let param = params.pop_back().unwrap();
            match param.value_type {
                ArgType::Bool => {}
                _ => {
                    return self.make_error(
                        Error::ParameterValueMissed,
                        format!("parametr \"{}\" has no value", param.name),
                    )
                }
            }
        };

        if let Some(cmd) = ctx.units.last() {
            let name = cmd.command.0;
            let cmd = cmd.command.1.clone();
            if let Some(f) = &cmd.exec {
                return Ok(f.borrow_mut()(ctx));
            } else {
                return self.make_error(
                    Error::CantExecuteCommand,
                    format!("No handler for command: {}", name),
                );
            }
        }

        Ok(Default::default())
    }

    fn make_error(&self, error: Error, details: String) -> Result<T::Result, Error> {
        if self.need_print_error {
            self.print_error(&error);
        }
        if self.need_print_help {
            let commands = self.commands();
            let buffer = format_help(commands);
            Cli::<T>::print_help(buffer.as_str());
        }
        Err(error)
    }

    fn print_error(&self, error: &crate::error::Error) {
        println!("{}", error)
    }

    pub(crate) fn print_help(buffer: &str) {
        println!("{}", buffer);
    }

    fn commands(&self) -> &HashMap<String, Rc<Command<T>>> {
        &self.root.1.subcommands
    }
}

/// `CliBuilder` is a helper using for build [`Cli`].
#[derive(Default, Debug)]
pub struct CliBuilder<T: Config> {
    commands: Vec<CommandBuilder<T>>,
    printer: Option<T::Printer>,
    need_print_error: bool,
    need_print_help: bool,
}

impl<T: Config> CliBuilder<T> {
    /// Add command
    pub fn command(mut self, cmd: CommandBuilder<T>) -> Self {
        self.commands.push(cmd);
        self
    }

    /// Switch output error message to stdout.
    pub fn print_error(mut self, enable: bool) -> Self {
        self.need_print_error = enable;
        self
    }

    /// Switch output help message to stdout.
    pub fn print_help(mut self, enable: bool) -> Self {
        self.need_print_help = enable;
        self
    }

    pub fn set_printer(mut self, printer: T::Printer) -> Self {
        self.printer = Some(printer);
        self
    }

    /// Build and return `Cli` object.
    pub fn build(mut self) -> Cli<T> {
        let mut commands = Default::default();

        while let Some(command_builder) = self.commands.pop() {
            add_command(&mut commands, command_builder, self.need_print_help);
        }

        if self.need_print_help {
            let cb = <CommandBuilder<T>>::with_name("help")
                .handler(crate::command::help_handler::<T>)
                .description("This help");

            add_command(&mut commands, cb, self.need_print_help);
        }

        Cli {
            root: (
                "root".to_owned(),
                Rc::new(Command {
                    subcommands: commands,
                    ..Default::default()
                }),
            ),
            printer: self.printer.unwrap_or_default(),
            need_print_help: self.need_print_help,
            need_print_error: self.need_print_error,
        }
    }
}

fn split_line(line: &str) -> impl Iterator<Item = (&str, Span)> {
    enum LineParseState {
        EndWord,
        StartWord { start: usize, quote: Option<char> },
    }

    let mut state = LineParseState::EndWord;
    let line_len = line.len();

    let result = line.char_indices().filter_map(move |(i, c)| {
        let mut result: Option<(&str, Span)> = None;
        if !c.is_whitespace() {
            match state {
                LineParseState::EndWord => {
                    let has_quote = c == '\'' || c == '\"';
                    state = LineParseState::StartWord {
                        start: if has_quote { i + 1 } else { i },
                        quote: if has_quote { Some(c) } else { None },
                    }
                }
                LineParseState::StartWord { start, quote } => {
                    if let Some(q) = quote {
                        if q == c {
                            result = Some((
                                &line[start..i],
                                Span {
                                    source: line,
                                    begin: start,
                                    end: i,
                                },
                            ));
                            state = LineParseState::EndWord;
                        }
                    }
                }
            }

            if line_len == i + 1 {
                if let LineParseState::StartWord { start, quote: _ } = state {
                    result = Some((
                        &line[start..],
                        Span {
                            source: line,
                            begin: start,
                            end: line_len,
                        },
                    ));
                }
            }
        } else {
            match state {
                LineParseState::EndWord => {}
                LineParseState::StartWord { start, quote } => {
                    if quote.is_none() {
                        result = Some((
                            &line[start..i],
                            Span {
                                source: line,
                                begin: start,
                                end: i,
                            },
                        ));
                        state = LineParseState::EndWord;
                    }
                }
            }
        }
        result
    });
    result
}

fn parse_arg(arg_type: ArgType, arg: &str) -> std::result::Result<ArgValue, String> {
    if arg.starts_with('-') && !(arg_type != ArgType::Int || arg_type != ArgType::Float) {
        return Err(format!("Seems {} is not a value", arg));
    }

    let value: ArgValue = match arg_type {
        ArgType::Bool => match arg {
            "true" | "yes" | "1" | "on" => ArgValue::Bool(true),
            "false" | "no" | "0" | "off" => ArgValue::Bool(false),
            _ => {
                return Err(format!(
                    "\"{}\" is not a boolean value. \
                                    Use \"1\", \"true\", \"yes\", \"on\" for true, \
                                    and \"0\", \"false\", \"no\", \"off\" for false",
                    arg
                ))
            }
        },

        ArgType::Int => match i64::from_str(arg) {
            Ok(i) => ArgValue::Int(i),
            Err(e) => return Err(format!("Parse int error: {}", e)),
        },

        ArgType::Float => match f64::from_str(arg) {
            Ok(f) => ArgValue::Float(f),
            Err(e) => return Err(format!("Parse float error: {}", e)),
        },

        ArgType::String => ArgValue::String(arg.to_string()),
    };

    Ok(value)
}

#[cfg(test)]
mod test {
    use crate::{ArgType, ArgValue};
    use assert2::check;

    macro_rules! check_arg {
        ($record:expr, $etalon:literal) => {
            let (arg, span) = &$record;
            check!(*arg == span.arg());
            check!(*arg == $etalon);
        };
    }

    #[test]
    fn split_line() {
        let line = "one two three";
        let v = super::split_line(line).collect::<Vec<_>>();
        check_arg!(v[0], "one");
        check_arg!(v[1], "two");
        check_arg!(v[2], "three");
    }

    #[test]
    fn split_line_with_quotes() {
        let line = "one \"two\" three";
        let v = super::split_line(line).collect::<Vec<_>>();
        check_arg!(v[0], "one");
        check_arg!(v[1], "two");
        check_arg!(v[2], "three");

        let line = "one \"two; two and half\" three";
        let v = super::split_line(line).collect::<Vec<_>>();
        check_arg!(v[0], "one");
        check_arg!(v[1], "two; two and half");
        check_arg!(v[2], "three");

        let line = "one two \"three\"";
        let v = super::split_line(line).collect::<Vec<_>>();
        check_arg!(v[0], "one");
        check_arg!(v[1], "two");
        check_arg!(v[2], "three");
    }

    #[test]
    fn split_line_with_bad_quotes() {
        let line = "one two \"three";
        let v = super::split_line(line).collect::<Vec<_>>();
        check_arg!(v[0], "one");
        check_arg!(v[1], "two");
        check_arg!(v[2], "three");
    }

    #[test]
    fn parse_arg_bool() {
        let f = |arg, state| {
            let result = super::parse_arg(ArgType::Bool, arg);
            match result {
                Ok(arg_value) => match arg_value {
                    ArgValue::Bool(v) => assert_eq!(v, state),
                    _ => panic!("bad type"),
                },
                Err(err) => panic!("{:?}", err),
            }
        };
        let true_args = ["true", "1", "yes", "on"];
        for arg in true_args.iter() {
            f(arg, true);
        }
        let false_args = ["false", "0", "no", "off"];
        for arg in false_args.iter() {
            f(arg, false);
        }
    }
    #[test]
    fn parse_arg_bool_error() {
        let result = super::parse_arg(ArgType::Bool, "not_a_bool");
        match result {
            Ok(arg_value) => panic!("error expected, but got this: {:?}", arg_value),
            Err(err) => assert!(!err.is_empty()),
        }
    }

    #[test]
    fn parse_arg_int() {
        use rand::prelude::*;

        let mut numbers = Vec::<(String, i64)>::with_capacity(12);
        numbers.push((i64::MIN.to_string(), i64::MIN));
        numbers.push((i64::MAX.to_string(), i64::MAX));
        for _ in 0..100 {
            let num: i64 = random();
            numbers.push((num.to_string(), num));
        }
        for (arg, state) in numbers.iter() {
            let result = super::parse_arg(ArgType::Int, arg.as_str());
            match result {
                Ok(arg_value) => match arg_value {
                    ArgValue::Int(v) => assert_eq!(v, *state),
                    _ => panic!("bad type"),
                },
                Err(err) => panic!("{:?}", err),
            }
        }
    }

    #[test]
    fn parse_arg_int_error() {
        let result = super::parse_arg(ArgType::Int, "not_int");
        match result {
            Ok(arg_value) => panic!("error expected, but got this: {:?}", arg_value),
            Err(err) => assert!(!err.is_empty()),
        }
    }

    #[test]
    fn parse_arg_float() {
        use rand::prelude::*;

        let mut numbers = Vec::<(String, f64)>::with_capacity(12);
        numbers.push((f64::MIN.to_string(), f64::MIN));
        numbers.push((f64::MAX.to_string(), f64::MAX));

        let mut rnd = thread_rng();
        for _ in 0..100 {
            let num: f64 = rnd.gen_range(f64::MIN / 2.0..f64::MAX / 2.0);
            numbers.push((num.to_string(), num));
        }
        for (arg, state) in numbers.iter() {
            let result = super::parse_arg(ArgType::Float, arg.as_str());
            match result {
                Ok(arg_value) => match arg_value {
                    ArgValue::Float(v) => {
                        println!("arg: {} == v: {}", arg, v);
                        assert_eq!(v, *state)
                    }
                    _ => panic!("bad type"),
                },
                Err(err) => panic!("{:?}", err),
            }
        }
    }

    #[test]
    fn parse_arg_float_error() {
        let result = super::parse_arg(ArgType::Float, "not_float");
        match result {
            Ok(arg_value) => panic!("error expected, but got this: {:?}", arg_value),
            Err(err) => assert!(!err.is_empty()),
        }
    }
}
