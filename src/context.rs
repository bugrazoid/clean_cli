use crate::traits::Config;

use super::{ArgValue, Command, Parameter};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Context<'a, T: Config> {
    pub(crate) units: Vec<ContextUnit<'a, T>>,
    pub(crate) printer: &'a T::Printer,
}

impl<'a, T: Config> Context<'a, T> {
    pub fn command_units(&self) -> &Vec<ContextUnit<'a, T>> {
        &self.units
    }

    pub fn printer(&self) -> &T::Printer {
        self.printer
    }
}

#[derive(Debug)]
pub struct ContextUnit<'a, T: Config> {
    pub(crate) command: (&'a str, Rc<Command<T>>),
    pub(crate) parameters: HashMap<String, (Rc<Parameter>, ArgValue)>,
    pub(crate) value: Option<ArgValue>,
}

impl<'a, T: Config> ContextUnit<'a, T> {
    pub fn name(&self) -> &'a str {
        self.command.0
    }

    pub fn parameters(&self) -> &HashMap<String, (Rc<Parameter>, ArgValue)> {
        &self.parameters
    }

    pub fn value(&self) -> Option<&ArgValue> {
        self.value.as_ref()
    }
}
