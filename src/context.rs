use super::{ArgValue, Command, Parameter};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Context<'a, R> {
    pub(super) units: Vec<ContextUnit<'a, R>>,
}

impl<'a, R> Context<'a, R> {
    pub fn command_units(&self) -> &Vec<ContextUnit<'a, R>> {
        &self.units
    }
}

#[derive(Debug)]
pub struct ContextUnit<'a, R> {
    pub(super) command: (&'a str, Rc<Command<R>>),
    pub(super) parameters: HashMap<String, (Rc<Parameter>, ArgValue)>,
    pub(super) value: Option<ArgValue>,
}

impl<'a, R> ContextUnit<'a, R> {
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
