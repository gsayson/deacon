//! Environment management and process execution.

use std::ffi::OsString;
use std::ops::Add;
use std::process::*;
use ansi_term::Colour::Red;
use lazy_static::lazy_static;
use regex::Regex;

/// Executes a process. Printing to the console is not done.
pub fn execute_process(input: impl ToString) -> Option<(Command, Child)> {
	let input = input.to_string();
	match shell_words::split(input.as_str()) {
		Ok(split) => {
			let split: Vec<String> = split;
			let mut iter = split.iter();
			let command_name = iter.next();
			let mut command = Command::new(command_name.unwrap());
			for i in iter {
				command.arg(i);
			}
			match command.stdout(Stdio::inherit()).stdin(Stdio::inherit()).spawn() {
				Ok(child) => {
					Some((command, child))
				}
				Err(err) => {
					eprintln!("{}", Red.paint(format!("Failed to execute \"{}\": {}", command_name.unwrap(), err.to_string())));
					None
				}
			}
		},
		Err(_) => {
			eprintln!("{}", Red.paint("Unclosed quotes."));
			None
		}
	}
}