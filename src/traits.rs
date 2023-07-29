use std::{collections::HashMap, fmt::Debug, marker::PhantomData, rc::Rc};

use crate::Command;

pub trait Config: Default + 'static {
    type Result: Default + Debug + 'static;
    type HelpFormatter: Formatter<Self>;
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

pub trait Formatter<T: Config> {
    fn format(commands: &HashMap<String, Rc<Command<T>>>) -> T::PrinterInput;
}

#[derive(Default)]
pub struct DefaultHelpFormatter;
impl<T: Config> Formatter<T> for DefaultHelpFormatter
where
    T::PrinterInput: From<String>,
{
    fn format(commands: &HashMap<String, Rc<Command<T>>>) -> T::PrinterInput {
        let mut buffer = "Help:".to_string();
        let mut keys: Vec<_> = commands.keys().collect();
        keys.sort();
        keys.iter().for_each(|key| {
            let cmd = commands.get(*key).expect("Existing key");
            let description = match cmd.description.as_ref() {
                Some(s) => s.as_str(),
                None => "",
            };
            buffer.push_str(format!("\n{:<20} {description}", key).as_str());
        });
        buffer.into()
    }
}
