//! This module implements information request commands.

use serde_json;

use std::collections::HashMap;

use super::{
    internal::{CollectedCommand, Command, Feedback},
    Argument, CommandName, Context,
};

// Context entry for the type of the requested information.
const INFO_REQUEST: &str = "INFO_REQUEST";

/// Information request command. Supported sub-commands:
///
/// - `core-version` - prints the version of the Exonum core.
/// - `list-services` - prints the list of the services the node is build with.
pub struct Info {
    pub services: Vec<String>,
}

impl Info {
    /// Creates a new `Info` instance.
    pub fn new(services: Vec<String>) -> Self {
        Self { services }
    }

    fn core_version() {
        println!("{}", option_env!("CARGO_PKG_VERSION").unwrap_or("?"));
    }

    fn list_services(&self) {
        let services = serde_json::to_string(&self.services).expect("Unable to serialize services");
        println!("{}", services);
    }
}

impl Command for Info {
    fn args(&self) -> Vec<Argument> {
        vec![Argument::new_named(
            INFO_REQUEST,
            true,
            "Request specific information.",
            "r",
            "request",
            false,
        )]
    }

    fn name(&self) -> CommandName {
        "info"
    }

    fn about(&self) -> &str {
        "Information request. Available actions: core-version, list-services."
    }

    fn execute(
        &self,
        _commands: &HashMap<CommandName, CollectedCommand>,
        context: Context,
        _: &dyn Fn(Context) -> Context,
    ) -> Feedback {
        let request = context
            .arg::<String>(INFO_REQUEST)
            .unwrap_or_else(|_| panic!("{} not found.", INFO_REQUEST));

        match request.as_ref() {
            "core-version" => Self::core_version(),
            "list-services" => self.list_services(),
            _ => println!("Unsupported information request: {}", request),
        }

        Feedback::None
    }
}
