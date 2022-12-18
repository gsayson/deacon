use std::collections::HashMap;

pub struct DeaconConfig {
	/// The contents of the upper line. Supports ANSI.
	pub upper_prompt: String,
	/// The contents of the Readline prompt. Does not support ANSI.
	pub lower_prompt: String,
	/// Integration configuration.
	pub integrations: Integrations,
	/// Whether to use Deacon's internal `dir` implementation.
	pub use_deacon_dir: bool,
	pub command_aliases: HashMap<String, String>
}

pub struct Integrations {
	pub rust_integration: bool
}