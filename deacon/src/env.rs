//! Environment management and process execution.

use std::process::*;
use ansi_term::Colour::Red;

/// Executes a process. Printing to the console is not done.
pub fn execute_process(input: impl ToString) -> Option<(Command, Child)> {
	let mut input = input.to_string();
	if input.starts_with("!") {
		input = input.replacen("!", "", 1);
	}
	input = input.replace("\\", "\\\\");
	input = input.replace("\\\\\"", "\\\"");
	input = input.replace("\\\\\\\\", "\\\\");
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

/// Substitutes environment variables into the new ones.
#[must_use]
pub fn substitute_env_var(input: impl AsRef<str>) -> String {
	let mut vec: Vec<String> = vec![];
	let input = input.as_ref();
	for s in input.split_whitespace() {
		let mut s = String::from(s);
		let s_clone = s.clone();
		if let Some(vars) = deacon_parse::parse_env_vars(&s_clone) {
			for var in vars {
				let var_representation = String::from("?") + var + "?";
				let val = std::env::var(&var).unwrap_or(var_representation.clone());
				s = s.replace(
					&var_representation,
					val.as_str()
				);
			}
		}
		vec.push(s);
	}
	vec.join(" ")
}

#[test]
fn test() {
    println!("{:?}", shell_words::split(r#"C:\s \" \\"#).unwrap());
}