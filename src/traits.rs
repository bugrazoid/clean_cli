use std::{collections::HashMap, fmt::Debug, marker::PhantomData, rc::Rc};

use crate::Command;

pub trait Config: Default + 'static {
    type Result: Default + Debug + 'static;
    type HelpFormatter: HelpFormatter<Self::Result>;
    type HelpPrinter: HelpPrinter<Self::Result, Self::HelpFormatter>;
}

pub struct DefaultConfig<R>(PhantomData<R>);
impl<R: Default + Debug + 'static> Config for DefaultConfig<R> {
    type Result = R;
    type HelpFormatter = DefaultHelpFormatter;
    type HelpPrinter = DefaultHelpPrinter;
}
impl<R> Default for DefaultConfig<R> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub trait HelpPrinter<R, HF: HelpFormatter<R>> {
    fn print(input: &HF::Output);
}

pub struct DefaultHelpPrinter;
impl<R, HF: HelpFormatter<R>> HelpPrinter<R, HF> for DefaultHelpPrinter
where
    HF::Output: std::fmt::Display,
{
    fn print(input: &HF::Output) {
        println!("{}", input);
    }
}

pub trait HelpFormatter<R> {
    type Output;
    fn format(commands: &HashMap<String, Rc<Command<R>>>) -> Self::Output;
}

pub struct DefaultHelpFormatter;
impl<R> HelpFormatter<R> for DefaultHelpFormatter {
    type Output = String;

    fn format(commands: &HashMap<String, Rc<Command<R>>>) -> Self::Output {
        let mut buffer = "Help:".to_string();
        commands.iter().for_each(|(key, cmd)| {
            let description = match cmd.description.as_ref() {
                Some(s) => s.as_str(),
                None => "",
            };
            buffer.push_str(format!("\n{:<20} {description}", key.as_str()).as_str());
        });
        buffer
    }
}
