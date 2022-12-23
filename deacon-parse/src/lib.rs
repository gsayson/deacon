#![feature(option_result_contains)]

pub mod function;
mod types;

use nom::character::complete::char;
use nom::error::{ErrorKind, ParseError};
use nom::{AsChar, InputTakeAtPosition, IResult};
use nom::multi::many1;
use nom::sequence::delimited;

/// Parses environment variables. If there is no environmental variable to substitute, this function will return [`None`].
/// Environment variables are delimited in `?`.
///
/// ```
/// # use deacon_parse::parse_env_vars;
/// assert_eq!(parse_env_vars("?"), None);
/// assert_eq!(parse_env_vars("??"), Some(vec![""]));
/// assert_eq!(parse_env_vars("???"), Some(vec![""]));
/// assert_eq!(parse_env_vars("?hi??"), Some(vec!["hi"]));
/// assert_eq!(parse_env_vars("?__??"), Some(vec!["__"]));
/// assert_eq!(parse_env_vars("?hi???"), Some(vec!["hi", ""]));
/// assert_eq!(parse_env_vars("?gamma_delta?"), Some(vec!["gamma_delta"]));
/// assert_eq!(parse_env_vars("????"), Some(vec!["", ""]));
/// assert_eq!(parse_env_vars("?foo?"), Some(vec!["foo"]));
/// assert_eq!(parse_env_vars("?space??bar?"), Some(vec!["space", "bar"]));
/// ```
pub fn parse_env_vars(input: &str) -> Option<Vec<&str>> {
	let res = many1(delimited(
		char('?'),
		alpha_underscore_0::<&str, ()>,
		char('?')
	))(input);
	match res {
		Ok((_, b)) => {
			Some(b)
		}
		Err(_) => {
			None
		}
	}
}

pub(crate) fn alpha_underscore_0<T: InputTakeAtPosition, E: ParseError<T>>(input: T) -> IResult<T, T, E> where <T as InputTakeAtPosition>::Item: AsChar {
	input.split_at_position_complete(|item| !({
		let ch = item.as_char();
		ch.is_alpha() || ch == '_'
	}))
}

pub(crate) fn alpha_underscore_1<T: InputTakeAtPosition, E: ParseError<T>>(input: T) -> IResult<T, T, E> where <T as InputTakeAtPosition>::Item: AsChar {
	input.split_at_position1_complete(|item| !({
		let ch = item.as_char();
		ch.is_alpha() || ch == '_'
	}), ErrorKind::Alpha)
}

#[cfg(test)]
mod tests {
    use super::*;
	use function::*;
	use types::*;
	use crate::types::DeaconType::String;

	#[test]
    fn parse_env_var() {
	    assert_eq!(parse_env_vars("?"), None);
	    assert_eq!(parse_env_vars("??"), Some(vec![""]));
	    assert_eq!(parse_env_vars("???"), Some(vec![""]));
	    assert_eq!(parse_env_vars("?hi??"), Some(vec!["hi"]));
	    assert_eq!(parse_env_vars("?__??"), Some(vec!["__"]));
	    assert_eq!(parse_env_vars("?hi???"), Some(vec!["hi", ""]));
	    assert_eq!(parse_env_vars("?gamma_delta?"), Some(vec!["gamma_delta"]));
	    assert_eq!(parse_env_vars("????"), Some(vec!["", ""]));
	    assert_eq!(parse_env_vars("?foo?"), Some(vec!["foo"]));
	    assert_eq!(parse_env_vars("?space??bar?"), Some(vec!["space", "bar"]));
    }

	#[test]
	fn parse_statements() {
		assert_eq!(
			parse_call("arbitrary_function $x $y"),
			Ok(Call {
				name: "arbitrary_function".to_string(),
				args: vec!["$x".to_string(), "$y".to_string()]
			})
		);
		assert_eq!(
			parse_call("arbitrary_function $x $y $z"),
			Ok(Call {
				name: "arbitrary_function".to_string(),
				args: vec!["$x".to_string(), "$y".to_string(), "$z".to_string()]
			})
		);
		assert_eq!(
			parse_call("not command_call $x $y $z"),
			Ok(Call {
				name: "not".to_string(),
				args: vec!["command_call".to_string(), "$x".to_string(), "$y".to_string(), "$z".to_string()]
			})
		);
	}

	#[test]
	fn parse_function_decls() {
	    assert!(parse_func_declaration("func a() {\necho $s\n}").is_ok());
		assert!(parse_func_declaration("func b(s: string) {\necho $s\n}").is_ok());
		assert!(parse_func_declaration("func c(s: string) {echo $s\n}").is_err());
		assert!(parse_func_declaration("func d(s: string) {\necho $s}").is_err());
		assert_eq!(parse_func_declaration("func e() {\necho $s\n}").unwrap().name, "b");
		assert_eq!(parse_func_declaration("func f() {\necho $s\n}").unwrap().name, "d");
	}

	#[test]
	fn parse_types() {
	    use types::DeaconType::*;
		assert!(DeaconType::try_from("str").is_err());
		assert_eq!(DeaconType::try_from("string"), Ok(String));
		assert_eq!(DeaconType::try_from("int"), Ok(Int));
		assert_eq!(DeaconType::try_from("(string, int)"), Ok(Tuple(vec![String, Int])));
		assert!(DeaconType::try_from("(string, int, (string, int))").is_err()); // tuples directly in a tuple are not supported.
	}

}