use super::command::*;
use super::context::*;
use super::error::*;
use super::parameter::*;

use std::{
    borrow::BorrowMut,
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
    str::FromStr,
    collections::{
        HashMap,
        VecDeque
    }
};

///  __Cli__ is a central unit that contains all possible commands, arguments and handlers.
/// To create instance using build pattern. Generic parameter using for return values from handlers.
/// 
/// # Example
///
/// ```rust
/// use clean_cli::*;
///
/// let cli = Cli::<bool>::builder()
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
pub struct Cli<R: Default> {
    data: Rc<RefCell<CliData<R>>>
}

#[derive(Debug)]
pub struct CliData<R: Default> {
    commands: HashMap<String, Rc<Command<R>>>,
    need_print_error: bool,
    need_print_help: bool,
}

enum ParseState {
    ReadFirst,
    ReadNext,
    ParametersReaded{ params: VecDeque<Rc<Parameter>> },
}

impl<R: Default> Cli<R> {
    /// Create builder
    pub fn builder() -> CliBuilder<R> {
        CliBuilder::default()
    }

    /// Execute _line_ 
    pub fn exec_line(&self, line: &str) -> Result<R, Error> {
        let mut ctx = Context::default();
        let mut state = ParseState::ReadFirst;
        let mut pos = 0;
        let args = split_line(line);

        for arg in args {
            match state {
                ParseState::ReadFirst => {
                    if arg.starts_with("--") || arg.starts_with("-") {
                        return self.make_error(Kind::CantExecuteParameter, arg.to_string());
                    } else if let Some(cmd) = (*self.data).borrow().commands.get(arg) {
                        ctx.units.push(ContextUnit {
                            command: (arg, cmd.clone()),
                            parameters: Default::default(),
                            value: None,
                        });
                        state = ParseState::ReadNext;
                    } else {
                        return self.make_error(Kind::NotCommand, arg.to_string());
                    }
                }

                ParseState::ReadNext => {
                    let last_unit = &mut ctx.units[pos];
                    let cmd = last_unit.command.1.clone();
                    let mut new_state: Option<ParseState> = None;

                    if arg.starts_with("--") {
                        let arg = &arg[2..];
                        if let Some(p) = cmd.parameters.get(arg) {
                            if let ArgType::Bool = p.value_type {
                                if let Some(p) = last_unit.command.1.parameters.get(arg) {
                                    last_unit.parameters.insert(p.name.clone(), (p.clone(), ArgValue::Bool(true)));
                                }
                            } else {
                                let mut params = VecDeque::with_capacity(1);
                                params.push_back(p.clone());
                                new_state = Some(ParseState::ParametersReaded{ params });
                            }
                        } else {
                            return self.make_error(Kind::NotParameter, arg.to_string());
                        }
                    } else if arg.starts_with("-") {
                        let arg = &arg[1..];
                        let mut params = VecDeque::with_capacity(arg.len());
                        for a in arg.chars() {
                            let s = a.to_string();
                            if let Some(p) = cmd.parameters.get(&s) {
                                if let ArgType::Bool = p.value_type {
                                    if let Some(p) = last_unit.command.1.parameters.get(&s) {
                                        last_unit.parameters.insert(p.name.clone(), (p.clone(), ArgValue::Bool(true)));
                                    }
                                }
                                else {
                                    params.push_back(p.clone());
                                }
                            } else {
                                return self.make_error(Kind::NotParameter, arg.to_string());
                            }
                        }

                        if !params.is_empty() {
                            new_state = Some(ParseState::ParametersReaded{
                                params
                            });
                        }
                    } else if let Some(v) = cmd.value.as_ref() {
                        match parse_arg(v.clone(), arg) {
                            Ok(value) => {
                                last_unit.value = Some(value);
                            }
                            Err(details) => return self.make_error(Kind::ValueParseFailed, details)
                        };
                    } else if let Some(sub) = cmd.subcommands.get(arg) {
                        ctx.units.push(ContextUnit {
                            command: (arg, sub.clone()),
                            parameters: Default::default(),
                            value: None,
                        });
                        pos += 1;
                        new_state = Some(ParseState::ReadNext);
                    } else {
                        return self.make_error(Kind::NotCommand, arg.to_string());
                    }

                    if let Some(s) = new_state {
                        state = s;
                    }
                }

                ParseState::ParametersReaded{ mut params} => {
                    let last_unit = &mut ctx.units[pos];

                    let param= params.pop_front().unwrap();
                    match parse_arg(param.value_type.clone(), arg) {
                        Ok(value) => {
                            last_unit.parameters.insert(param.name.clone(), (param.clone(), value));
                            if params.is_empty() {
                                state = ParseState::ReadNext;
                            } else {
                                state = ParseState::ParametersReaded{ params };
                            }
                        }
                        Err(details) => {
                            return self.make_error(Kind::ValueParseFailed, details);
                        }
                    };
                }
            }
        }

        if let ParseState::ParametersReaded{mut params} = state {
            if params.len() > 1 {
                return self.make_error(Kind::ParserError, format!("Wrong params value: {}", params.len()));
            }
            let param = params.pop_back().unwrap();
            match param.value_type {
                ArgType::Bool => {}
                _ => return self.make_error(Kind::ParameterValueMissed, format!("parametr \"{}\" has no value", param.name))
            }
        };

        if let Some(cmd) = ctx.units.last() {
            let cmd = cmd.command.1.clone();
            if let Some(f) = &cmd.exec {
                return Ok(f.borrow_mut()(ctx));
            } else {
                return self.make_error(Kind::CantExecuteCommand, format!(""));
            }
        }

        Ok(Default::default())
    }

    fn make_error(&self, kind: Kind, details: String) -> Result<R, Error> {
        let error = Error { kind, details };
        if (*self.data).borrow().need_print_error {
            self.print_error(&error);
        }
        if (*self.data).borrow().need_print_help {
            let commands = &self.data.borrow().commands;
            let buffer = format_help(commands);
            Cli::<R>::print_help(buffer.as_str());
        }
        Err(error)
    }

    fn print_error(&self, error: &crate::error::Error) {
        println!("{}", error)
    }

    fn print_help(buffer: &str) {
        println!("{}", buffer);
    }
}

/// **CliBuilder** is a helper using for build `Cli`.
#[derive(Default, Debug)]
pub struct CliBuilder<R> {
    commands: Vec<CommandBuilder<R>>,
    need_print_error: bool,
    need_print_help: bool,
}

impl<R: Default + 'static> CliBuilder<R> {
    /// Add command
    pub fn command(mut self, cmd: CommandBuilder<R>) -> Self {
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

    /// Build and return **Cli** object.
    pub fn build(self) -> Cli<R> {
        let mut commands = <HashMap<String, Rc<Command<R>>>>::default();

        let mut command_builders = self.commands;
        while let Some(command_builder) = command_builders.pop() {
            add_command(&mut commands, command_builder, self.need_print_help);
        }

        let cli_data = Rc::new(RefCell::new(CliData {
            commands,
            need_print_error: self.need_print_error,
            need_print_help: self.need_print_help,
        }));

        if self.need_print_error {
            let cd = cli_data.clone();
            let cb = CommandBuilder::with_name("help")
                .handler(move |_| {
                    let commands = &cd.borrow().commands;
                    let help_text = format_help(commands);
                    println!("{}", help_text);
                    R::default()
                })
                .description("This help");

            add_command((*cli_data).borrow_mut().commands.borrow_mut(), cb, self.need_print_help);
        }

        Cli{ data: cli_data }
    }
}

enum LineParseState {
    EndWord,
    StartWord{start: usize, quote: Option<char>}
}

fn split_line(line: &str) -> impl Iterator<Item = &str> {
    let mut state = LineParseState::EndWord;
    let line_len = line.len();
    let result = line.char_indices()
        .map(move |(i, c)| {
            let mut result: Option<_> = None;
            if !c.is_whitespace() {
                match state {
                    LineParseState::EndWord => {
                        let has_quote = c == '\'' || c == '\"';
                        state = LineParseState::StartWord{
                            start: if has_quote { i+1 } else { i }, 
                            quote: if has_quote { Some(c) } else { None }
                        }
                    }
                    LineParseState::StartWord { start, quote } => {
                        if let Some(q) = quote {
                            if q == c {
                                result = Some(&line[start..i]);
                                state = LineParseState::EndWord;
                            }
                        }
                    },
                }

                if line_len == i+1 {
                    if let LineParseState::StartWord { start, quote: _ } = state {
                        result = Some(&line[start..]);
                    }
                }
            } else {
                match state {
                    LineParseState::EndWord => {},
                    LineParseState::StartWord { start, quote } => {
                        if quote.is_none() {
                            result = Some(&line[start..i]);
                            state = LineParseState::EndWord;
                        }
                    },
                }
            }
            result
        })
        .filter(|x| x.is_some())
        .map(|s| s.unwrap());
    result
}

fn parse_arg(arg_type: ArgType, arg: &str) -> std::result::Result<ArgValue, String> {
    if arg.starts_with("-") && !(arg_type != ArgType::Int || arg_type != ArgType::Float) {
        return Err(format!("Seems {} is not a value", arg));
    }

    let value: ArgValue = match arg_type {
        ArgType::Bool => match arg {
            "true" | "yes" | "1" | "on"  => ArgValue::Bool(true),
            "false" | "no" | "0" | "off" => ArgValue::Bool(false),
            _ => return Err(format!("\"{}\" is not a boolean value. \
                                    Use \"1\", \"true\", \"yes\", \"on\" for true, \
                                    and \"0\", \"false\", \"no\", \"off\" for false", arg))
        }

        ArgType::Int => {
            match i64::from_str(arg) {
                Ok(i) => ArgValue::Int(i),
                Err(e) => return Err(format!("Parse int error: {}", e))
            }
        }

        ArgType::Float => {
            match f64::from_str(arg) {
                Ok(f) => ArgValue::Float(f),
                Err(e) => return Err(format!("Parse float error: {}", e))
            }
        }

        ArgType::String => ArgValue::String(arg.to_string()),
    };

    Ok(value)
}

#[cfg(test)]
mod test {
    use crate::{ArgType, ArgValue, Cli, CommandBuilder, Parameter};

    #[test]
fn print_help() {
    let cli = Cli::<()>::builder()
    .print_help(true)
    .command(CommandBuilder::with_name("cmd")
        .parameter(Parameter::with_name("bool")
            .value_type(ArgType::Bool)
            .alias("b")
            .alias("bb")
            .description("Some about bool")
        )
        .parameter(Parameter::with_name("int")
            .value_type(ArgType::Int)
            .alias("i")
            .alias("ii")
            .description("Some about int")
        )
        .parameter(Parameter::with_name("float")
            .value_type(ArgType::Float)
            .alias("f")
            .alias("ff")
            .description("Some about float")
        )
        .parameter(Parameter::with_name("string")
            .value_type(ArgType::String)
            .alias("s")
            .alias("ss")
            .description("Some about string")
        )
        .description("Main command for all other commands")
        .use_value(ArgType::Bool)
    )
    .command(CommandBuilder::with_name("other_cmd")
        .description("Some other command")
        .use_value(ArgType::Bool))
    .build();

    // cli.exec_line("help").unwrap();
    assert!(cli.exec_line("bad").is_err());
    assert!(true)
}

    #[test]
    fn split_line() {
        let line = "one two three";
        let v = super::split_line(line).collect::<Vec<&str>>();
        assert_eq!(v[0], "one");
        assert_eq!(v[1], "two");
        assert_eq!(v[2], "three");
    }
    
    #[test]
    fn split_line_with_quotes() {
        let line = "one \"two\" three";
        let v = super::split_line(line).collect::<Vec<&str>>();
        assert_eq!(v[0], "one");
        assert_eq!(v[1], "two");
        assert_eq!(v[2], "three");

        let line = "one \"two; two and half\" three";
        let v = super::split_line(line).collect::<Vec<&str>>();
        assert_eq!(v[0], "one");
        assert_eq!(v[1], "two; two and half");
        assert_eq!(v[2], "three");

        let line = "one two \"three\"";
        let v = super::split_line(line).collect::<Vec<&str>>();
        assert_eq!(v[0], "one");
        assert_eq!(v[1], "two");
        assert_eq!(v[2], "three");
    }

    #[test]
    fn split_line_with_bad_quotes() {
        let line = "one two \"three";
        let v = super::split_line(line).collect::<Vec<&str>>();
        assert_eq!(v[0], "one");
        assert_eq!(v[1], "two");
        assert_eq!(v[2], "three");
    }

    #[test]
    fn parse_arg_bool() {
        let f = |arg, state| {
            let result = super::parse_arg(ArgType::Bool, arg);
            match result {
                Ok(arg_value) => match arg_value {
                    ArgValue::Bool(v) => assert_eq!(v, state),
                    _ => panic!("bad type")
                }
                Err(err) => panic!("{:?}", err)
            }
        };
         let true_args = ["true", "1", "yes", "on"];
        for arg in true_args.iter() { f(arg, true); }
         let false_args = ["false", "0", "no", "off"];
        for arg in false_args.iter() { f(arg, false); }
    }
     #[test]
    fn parse_arg_bool_error() {
        let result = super::parse_arg(ArgType::Bool, "not_a_bool");
        match result {
            Ok(arg_value) => panic!("error expected, but got this: {:?}", arg_value),
            Err(err) => assert!(!err.is_empty())
        }
    }

    #[test]
    fn parse_arg_int() {
        use rand::prelude::*;

        let mut numbers = Vec::<(String,i64)>::with_capacity(12);
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
                    _ => panic!("bad type")
                }
                Err(err) => panic!("{:?}", err)
            }
        }
    }

    #[test]
    fn parse_arg_int_error() {
        let result = super::parse_arg(ArgType::Int, "not_int");
        match result {
            Ok(arg_value) => panic!("error expected, but got this: {:?}", arg_value),
            Err(err) => assert!(!err.is_empty())
        }
    }

    #[test]
    fn parse_arg_float() {
        use rand::prelude::*;

        let mut numbers = Vec::<(String,f64)>::with_capacity(12);
        numbers.push((f64::MIN.to_string(), f64::MIN));
        numbers.push((f64::MAX.to_string(), f64::MAX));

        let mut rnd = thread_rng();
        for _ in 0..100 {
            let num: f64 = rnd.gen_range(f64::MIN .. f64::MAX);
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
                    _ => panic!("bad type")
                }
                Err(err) => panic!("{:?}", err)
            }
        }
    }

    #[test]
    fn parse_arg_float_error() {
        let result = super::parse_arg(ArgType::Float, "not_float");
        match result {
            Ok(arg_value) => panic!("error expected, but got this: {:?}", arg_value),
            Err(err) => assert!(!err.is_empty())
        }
    }
}