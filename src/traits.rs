use std::{collections::HashMap, fmt::Debug, marker::PhantomData, rc::Rc};

use crate::Command;

pub trait Config: Default + 'static {
    type Result: Default + Debug + 'static;
    type Formatter: Formatter<Self>;
    type Printer: Printer<Self, Self::Formatter> + Default;
}

pub struct DefaultConfig<R>(PhantomData<R>);
impl<R: Default + Debug + 'static> Config for DefaultConfig<R> {
    type Result = R;
    type Formatter = DefaultFormatter;
    type Printer = DefaultPrinter;
}
impl<R> Default for DefaultConfig<R> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub trait Printer<T: Config, HF: Formatter<T>> {
    fn print(&self, input: HF::Output);
}

#[derive(Default)]
pub struct DefaultPrinter;
impl<T: Config, HF: Formatter<T>> Printer<T, HF> for DefaultPrinter
where
    HF::Output: std::fmt::Display,
{
    fn print(&self, input: HF::Output) {
        println!("{}", input);
    }
}

pub trait Formatter<T: Config> {
    type Output;
    fn format(commands: &HashMap<String, Rc<Command<T>>>) -> Self::Output;
}

#[derive(Default)]
pub struct DefaultFormatter;
impl<T: Config> Formatter<T> for DefaultFormatter {
    type Output = String;

    fn format(commands: &HashMap<String, Rc<Command<T>>>) -> Self::Output {
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
        buffer
    }
}
