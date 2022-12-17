pub struct DeaconConfig {
	// The contents of the upper line. Supports ANSI.
	pub upper_prompt: String,
	// The contents of the Readline prompt. Does not support ANSI.
	pub lower_prompt: String,
	/// Integration configuration.
	pub integrations: Integrations,
}

pub struct Integrations {
	pub rust_integration: bool,
	pub git_integration: bool,
	pub java_integration: bool,
}