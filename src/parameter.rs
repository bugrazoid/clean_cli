/// Contains value for commands and parameters
#[derive(Debug, Clone)]
pub enum ArgValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

/// Set value type for commands and parameters
#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    Bool,
    Int,
    Float,
    String,
}

impl Default for ArgType {
    fn default() -> Self {
        ArgType::Bool
    }
}

/// Command parameter
#[derive(Debug)]
pub struct Parameter {
    pub(crate) name: String,
    pub(crate) value_type: ArgType,
    pub(crate) description: String,
}

/// Buildr for command parameter
#[derive(Default)]
pub struct ParameterBuilder<'a> {
    pub(crate) name: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) description: Option<&'a str>,
    pub(crate) value_type: ArgType,
}

impl Parameter {
    /// Create parameter builder with name
    pub fn with_name(name: &str) -> ParameterBuilder {
        ParameterBuilder {
            name: name.to_string(),
            ..Default::default()
        }
    }
}

impl<'a> ParameterBuilder<'a> {
    /// Set parameters value type
    pub fn value_type(mut self, t: ArgType) -> Self {
        self.value_type = t;
        self
    }

    /// Add alias for parameter.
    /// If alias consist from single charceter than this parameter will be used as short
    pub fn alias(mut self, alias: &'a str) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add description that shown in help
    pub fn description(mut self, text: &'a str) -> Self {
        self.description = Some(text);
        self
    }
}
