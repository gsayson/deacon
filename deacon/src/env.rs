//! Environment management and process execution.

use std::ops::Range;
use std::process::*;
use ansi_term::Colour::Red;
use ariadne::{FnCache, Label, ReportKind, Source};

/// Executes a process. Printing to the console is not done.
pub fn execute_process(input: impl ToString) -> Option<(Command, Child)> {
	let mut input = input.to_string();
	let input_original = input.clone();
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
			if iter.is_empty() {
				// there is nothing; this is therefore invalid syntax
				ariadne::Report::build(ReportKind::Error, (), input_original.find("!").unwrap())
					.with_code(1)
					.with_message("Exclamation-mark builtin escape syntax is used without any process name")
					.with_help("Include the name of the process you want to execute after the exclamation mark.")
					.with_note("If you wanted to have a if-not statement, use the `not` builtin command instead.")
					.with_label(Label::new(input_original.find("!").unwrap()..(input_original.find("!").unwrap() + 1)).with_message("The exclamation mark is used by itself"))
					.finish()
					.eprint(Source::from(input_original))
					.unwrap_or(());
				return None;
			}
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