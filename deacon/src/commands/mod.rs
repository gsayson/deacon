//! Internal commands.

use ansi_term::Colour::*;
use crate::util::print_help;

// input is guaranteed to NOT be blank.
pub fn resolve_function(input: impl AsRef<str>) -> bool {
	let input = input.as_ref();
	match input.split_whitespace().next().unwrap() {
		"cd" => change_dir(input),
		"devconinfo" => print_devcon_info(),
		"help" => print_help(),
		&_ => {
			return false;
		}
	}
	return true
}

pub fn change_dir(input: impl AsRef<str>) {
	let input = input.as_ref();
	let path = input.split_whitespace().skip(1).next();
	if let Some(path) = path {
		match std::env::set_current_dir(path) {
			Ok(_) => {}
			Err(err) => {
				eprintln!("Failed to change directory: {}", err.to_string());
			}
		}
	} else {
		eprintln!("Please provide a path.");
	}
}

pub fn print_devcon_info() {
	println!("{} {} {}",
	         Yellow.bold().paint("DevCon"),
	         Cyan.paint(env!("CARGO_PKG_VERSION")),
	         {
		         if !env!("CARGO_PKG_VERSION").starts_with("0.") {
			         Green.bold().paint("stable")
		         } else {
			         Red.bold().paint("unstable")
		         }
	         },
	);
	println!("debug build: {}", crate::util::colorize_bool(cfg!(debug_assertions)));
}