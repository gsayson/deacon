use ansi_term::ANSIGenericString;
use ansi_term::Colour::*;

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