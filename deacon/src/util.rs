use ansi_term::ANSIGenericString;
use ansi_term::Colour::*;
use comfy_table::*;
use comfy_table::presets::UTF8_FULL;
use lazy_static::lazy_static;

pub fn colorize_bool(boolean: bool) -> ANSIGenericString<'static, str> {
	if boolean {
		Green.bold().paint("true")
	} else {
		Red.bold().paint("false")
	}
}

pub fn print_prompt() {
	use crate::integrations::rust::*;
	print!(
		"{}: {} ",
		Green.bold().paint(format!("{}@{}", whoami::username(), whoami::hostname())),
		Blue.bold().paint(std::env::current_dir().unwrap().to_string_lossy().trim_end())
	);

	// start of integrations

	// rust integration
	if is_rust_project(std::env::current_dir().unwrap()) {
		print!("{} ", RGB(255, 165, 0).paint({
			if is_cargo_workspace(std::env::current_dir().unwrap()) {
				"[cargo workspace]".to_string()
			} else {
				let manifest = get_crate_manifest(std::env::current_dir().unwrap());
				if let Some(manifest) = manifest {
					if let Some(package) = manifest.package {
						format!("[crate \"{}\"]", package.name())
					} else {
						"[rust project]".to_string()
					}
				} else {
					"[rust project]".to_string()
				}
			}
		}))
	}

	// end of integrations

	println!();
}

lazy_static! {
	static ref HELP_TABLE: Table = {
		let mut table = Table::new();
	table
		.load_preset(UTF8_FULL)
		.set_content_arrangement(ContentArrangement::Dynamic)
		.set_header(vec!["Command", "Description", "Example"])
		.add_row(vec![
			"help",
			"Print help information.",
			"help"
		]).add_row(vec![
			"cd <path>",
			"Change directory to the given path.",
			"cd /"
		]).add_row(vec![
			"dir (path)",
			"List the given directory's files. If a directory is not provided, it defaults to the current working directory.",
			"cd /"
		]);
		table
	};
}

pub fn print_help() {
	println!("{}", HELP_TABLE.to_string());
}