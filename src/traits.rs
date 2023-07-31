use crate::{Command, Parameter};
use std::{fmt::Debug, marker::PhantomData};

pub trait Config: Default + 'static {
    type Result: Default + Debug + 'static;
    type HelpFormatter: HelpFormatter<Self>;
    type PrinterInput;
    type Printer: Printer<Self> + Default;
}

pub struct DefaultConfig<R>(PhantomData<R>);
impl<R: Default + Debug + 'static> Config for DefaultConfig<R> {
    type Result = R;
    type HelpFormatter = DefaultHelpFormatter;
    type PrinterInput = String;
    type Printer = DefaultPrinter;
}
impl<R> Default for DefaultConfig<R> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub trait Printer<T: Config> {
    fn print(&self, input: T::PrinterInput);
}

#[derive(Default)]
pub struct DefaultPrinter;
impl<T: Config> Printer<T> for DefaultPrinter
where
    T::PrinterInput: std::fmt::Display,
{
    fn print(&self, input: T::PrinterInput) {
        println!("{}", input);
    }
}

pub trait HelpFormatter<T: Config> {
    fn format(commands: &Command<T>) -> T::PrinterInput;
}

#[derive(Default)]
pub struct DefaultHelpFormatter;
impl<T: Config> HelpFormatter<T> for DefaultHelpFormatter
where
    T::PrinterInput: From<String>,
{
    fn format(command: &Command<T>) -> T::PrinterInput {
        use std::collections::{BTreeMap, BTreeSet};

        const TAB0: usize = 2;
        const TAB1: usize = 4;
        let mut delimiter = false;
        let mut buffer = "Help:".to_string();

        let params = &command.parameters;
        if !params.is_empty() {
            delimiter = true;
            buffer.push_str(format!("\n{:TAB0$}Parameters:", "").as_str());

            let mut aliases: BTreeMap<&str, (&Parameter, BTreeSet<&str>)> = Default::default();
            for (key, param) in params.iter() {
                if let Some((_, vec)) = aliases.get_mut(param.name.as_str()) {
                    if *key != param.name {
                        vec.insert(key);
                    }
                } else {
                    aliases.insert(
                        &param.name,
                        (
                            param.as_ref(),
                            if *key == param.name {
                                BTreeSet::new()
                            } else {
                                BTreeSet::from([key.as_str()])
                            },
                        ),
                    );
                }
            }

            aliases.iter_mut().for_each(|(name, (param, aliases))| {
                let mut a = (if name.len() > 1 { "--" } else { "-" }).to_string() + name;
                for n in aliases.iter() {
                    a.push(',');
                    if a.len() + n.len() >= 20 {
                        a.push('\n');
                    }
                    a.push_str(if n.len() > 1 { "--" } else { "-" });
                    a.push_str(n);
                }
                buffer.push_str(
                    format!(
                        "\n{:TAB1$}{:20}{:8}{}",
                        "",
                        a,
                        format!("<{}>", param.value_type),
                        param.description
                    )
                    .as_str(),
                );
            });
        }

        let commands = &command.subcommands;
        if !commands.is_empty() {
            if delimiter {
                buffer.push_str("\n----------------------------------------");
            }
            buffer.push_str(format!("\n{:TAB0$}Subcommands:", "").as_str());
            let keys: BTreeSet<_> = commands.keys().collect();
            keys.iter().for_each(|key| {
                let cmd = commands.get(*key).expect("Command not found");
                let description = match cmd.description.as_ref() {
                    Some(s) => s.as_str(),
                    None => "",
                };
                buffer.push_str(format!("\n{:TAB1$}{key:<20} {description}", "").as_str());
            });
        }

        buffer.into()
    }
}
