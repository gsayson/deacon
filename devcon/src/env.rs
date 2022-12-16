//! Environment management and process execution.

use std::process::*;
use ansi_term::Colour::Red;
use duct::*;

/// Executes a process. Printing to the console is not done.
pub fn execute_process(input: impl ToString) -> Option<(Command, Child)> {
	let input = input.to_string();
	let mut iter = input.split_ascii_whitespace();
	let command_name = iter.next();
	let mut command = Command::new(command_name.unwrap());
	for i in iter {
		command.arg(i);
	}
	match command.stdout(Stdio::inherit()).stdin(Stdio::piped()).spawn() {
		Ok(mut child) => {
			Some((command, child))
		}
		Err(err) => {
			eprintln!("{}", Red.paint(format!("Failed to execute \"{}\": {}", command_name.unwrap(), err.to_string())));
			None
		}
	}
}