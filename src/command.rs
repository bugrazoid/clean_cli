use crate::traits;

use super::{context::Context, parameter::*};

use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Formatter},
    rc::Rc,
};

const NO_DESCRIPTION: &str = "";

type CallBack<R> = RefCell<Box<dyn FnMut(Context<R>) -> R>>;

/// `CommandBuilder` is a helper using for build [`Command`].
#[derive(Default)]
pub struct CommandBuilder<R> {
    name: String,
    aliases: Vec<String>,
    subcommands: Vec<CommandBuilder<R>>,
    value: Option<ArgType>,
    description: Option<String>,
    parameters: HashMap<String, Rc<Parameter>>,
    handler: Option<CallBack<R>>,
}

/// `Command` stores all associated options, subcommands, values, and handler.
#[derive(Default)]
pub struct Command<R> {
    pub(super) subcommands: HashMap<String, Rc<Command<R>>>,
    pub(super) value: Option<ArgType>,
    pub(super) description: Option<String>,
    pub(super) parameters: HashMap<String, Rc<Parameter>>,
    pub(super) exec: Option<CallBack<R>>,
}

impl<R: Default + Debug + 'static> CommandBuilder<R> {
    /// Create command with name.
    pub fn with_name(name: &str) -> CommandBuilder<R> {
        CommandBuilder {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Add alias for command.
    pub fn alias(mut self, alias: &str) -> Self {
        self.aliases.push(alias.to_string());
        self
    }

    /// Add subcommand
    /// # Panic
    /// Panics if command has no executor or command with same name already exist
    pub fn subcommand(mut self, command: CommandBuilder<R>) -> Self {
        self.subcommands.push(command);
        self
    }

    /// Add parameter
    pub fn parameter(mut self, param: ParameterBuilder) -> Self {
        add_parameter(self.parameters.borrow_mut(), param);
        self
    }

    /// Set command handler
    pub fn handler<F>(mut self, f: F) -> Self
    where
        F: FnMut(Context<R>) -> R + 'static,
    {
        self.handler = Some(RefCell::new(Box::new(f)));
        self
    }

    /// Set command value type if required.
    pub fn use_value(mut self, value_type: ArgType) -> Self {
        self.value = Some(value_type);
        self
    }

    pub fn description(mut self, text: &str) -> Self {
        self.description = Some(text.to_owned());
        self
    }

    fn build(self, need_print_help: bool) -> (Command<R>, String, Vec<String>) {
        if self.value.is_none() && self.handler.is_none() && self.subcommands.is_empty() {
            panic!(
                "command \"{}: {}\" has no value or handler or subcommand",
                &self.name,
                self.description.unwrap_or(NO_DESCRIPTION.to_string())
            )
        }

        (
            Command {
                subcommands: Self::build_subcommands(self.subcommands, need_print_help),
                value: self.value,
                description: self.description,
                parameters: self.parameters,
                exec: self.handler,
            },
            self.name,
            self.aliases,
        )
    }

    fn build_subcommands(
        subcommands: Vec<CommandBuilder<R>>,
        need_print_help: bool,
    ) -> HashMap<String, Rc<Command<R>>> {
        let mut subcommands_builders = subcommands;
        let mut commands = <HashMap<String, Rc<Command<R>>>>::default();
        let sub_count = subcommands_builders.len();

        while let Some(command_builder) = subcommands_builders.pop() {
            add_command(&mut commands, command_builder, need_print_help);
        }

        if need_print_help && sub_count > 0 {
            let cb = CommandBuilder::with_name("help")
                .handler(
                    help_handler::<R, traits::DefaultHelpFormatter, traits::DefaultHelpPrinter>,
                )
                .description("This help");

            add_command(&mut commands, cb, need_print_help);
        }

        commands
    }
}

pub(super) fn format_help<R: Default>(commands: &HashMap<String, Rc<Command<R>>>) -> String {
    let mut buffer = "Help:".to_string();
    commands.iter().for_each(|(key, cmd)| {
        let description = match cmd.description.as_ref() {
            Some(s) => s.as_str(),
            None => "",
        };
        buffer.push_str(format!("\n{:<20}| {description}", key.as_str()).as_str());
    });
    buffer
}

pub(super) fn help_handler<R, HF, HP>(ctx: Context<R>) -> R
where
    R: Default + Debug + 'static,
    HF: traits::HelpFormatter<R>,
    HP: traits::HelpPrinter<R, HF>,
{
    let last = ctx.command_units().len().saturating_sub(1);
    let commands = &ctx.command_units()[last.saturating_sub(1)]
        .command
        .1
        .subcommands;
    let buffer = HF::format(commands);
    HP::print(&buffer);
    R::default()
}

pub(super) fn add_command<R: Default + Debug + 'static>(
    commands: &mut HashMap<String, Rc<Command<R>>>,
    command_builder: CommandBuilder<R>,
    need_print_help: bool,
) {
    if let Some(exist) = commands.get(&command_builder.name) {
        panic!(
            "command \"{}: {}\" already exist\nand can not be replaced with command \"{}: {}\"",
            &command_builder.name,
            exist
                .description
                .as_ref()
                .unwrap_or(&NO_DESCRIPTION.to_string()),
            &command_builder.name,
            command_builder
                .description
                .unwrap_or(NO_DESCRIPTION.to_string())
        );
    }

    let (command, name, mut aliases) = command_builder.build(need_print_help);
    let command = Rc::new(command);
    commands.insert(name, command.clone());
    while let Some(alias) = aliases.pop() {
        commands.insert(alias, command.clone());
    }
}

fn add_parameter(
    parameters: &mut HashMap<String, Rc<Parameter>>,
    parameter_builder: ParameterBuilder,
) {
    if let Some(_) = parameters.get(&parameter_builder.name) {
        panic!(
            "parameter with name \"{}\" already exist",
            &parameter_builder.name
        );
    }

    let parameter = Rc::new(Parameter {
        name: parameter_builder.name.clone(),
        value_type: parameter_builder.value_type,
        description: parameter_builder.description.unwrap_or("").into(),
    });

    parameters.insert(parameter_builder.name.into(), parameter.clone());
    let mut aliases = parameter_builder.aliases;
    while let Some(alias) = aliases.pop() {
        parameters.insert(alias, parameter.clone());
    }
}

impl<R> std::fmt::Debug for self::Command<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(Command))
            .field("value", &self.value)
            .field("options", &self.parameters)
            .field("subcommands", &self.subcommands)
            .field("description", &self.description)
            .finish()
    }
}

impl<R> std::fmt::Debug for self::CommandBuilder<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(CommandBuilder))
            .field("value", &self.value)
            .field("options", &self.parameters)
            .field("subcommands", &self.subcommands)
            .field("description", &self.description)
            .finish()
    }
}

impl<R> Command<R> {}
