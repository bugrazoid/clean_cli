#[derive(Debug, Clone)]
pub enum ArgValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    Bool,
    Int,
    Float,
    String
}

impl Default for ArgType {
    fn default() -> Self {
        ArgType::Bool
    }
}

#[derive(Debug)]
pub struct Parameter {
    pub(super) name: String,
    pub(super) value_type: ArgType,
    pub(super) description: String
}

#[derive(Default)]
pub struct ParameterBuilder<'a> {
    pub(super) name: String,
    pub(super) aliases: Vec<String>,
    pub(super) description: Option<&'a str>,
    pub(super) value_type: ArgType
}

impl Parameter {
    pub fn with_name(name: &str) -> ParameterBuilder {
        ParameterBuilder {
            name: name.to_string(),
            ..
            Default::default()
        }
    }
}

impl<'a> ParameterBuilder<'a> {
    pub fn value_type(mut self, t: ArgType) -> Self {
        self.value_type = t;
        self
    }

    pub fn alias(mut self, alias: &'a str) -> Self {
        self.aliases.push(alias.into());
        self
    }

    pub fn description(mut self, text: &'a str) -> Self {
        self.description = Some(text);
        self
    }
}
