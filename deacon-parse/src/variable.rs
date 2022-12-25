//! Standard variable parsing.
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, multispace1};
use nom::character::streaming::multispace0;
use nom::error::VerboseError;
use nom::sequence::tuple;
use crate::alpha_underscore_1;

/// Parse a variable declaration in the form:
/// ```sh
/// 'let' WHITESPACE '$' IDENTIFIER '=' value
/// ```
/// Note that if there was a previous variable by the given name in the given declaration's
/// current scope, the value of that previous variable is freed to save memory. To modify a
/// variable instead, one can [reassign](self::parse_variable_reassignment) the variable
/// to another value instead.
pub fn parse_variable_decl(input: &str) -> Result<Variable, VerboseError<&str>> {
	//..................... .let..........$    idf..........=..........val.//
	let res: Result<(&str, (&str, &str, char, &str, &str, char, &str, &str)), VerboseError<&str>> = tuple((
		tag::<_, &str, VerboseError<&str>>("let"),
		multispace1::<&str, VerboseError<&str>>,
		char::<&str, VerboseError<&str>>('$'),
		alpha_underscore_1::<&str, VerboseError<&str>>,
		multispace0::<&str, VerboseError<&str>>,
		char::<&str, VerboseError<&str>>('='),
		multispace0::<&str, VerboseError<&str>>,
		take_while1::<_, &str, VerboseError<&str>>(|ch| ch != '\n')
	))(input).map_err(crate::MAP_ERR);
	let var = res?.1.to_owned();
	Ok(Variable {
		identifier: var.3.trim().to_string(),
		value: var.7.trim().to_string()
	})
}

/// Parse a variable reassignment in the form:
/// ```sh
/// WHITESPACE '$' IDENTIFIER '=' value
/// ```
/// Note that this totally differs from [variable declaration](self::parse_variable_decl), which frees old variables
/// of the same name in the same scope.
pub fn parse_variable_reassignment(input: &str) -> Result<Variable, VerboseError<&str>> {
	//........................$    idf..........=..........val.//
	let res: Result<(&str, (char, &str, &str, char, &str, &str)), VerboseError<&str>> = tuple((
		char::<&str, VerboseError<&str>>('$'),
		alpha_underscore_1::<&str, VerboseError<&str>>,
		multispace0::<&str, VerboseError<&str>>,
		char::<&str, VerboseError<&str>>('='),
		multispace0::<&str, VerboseError<&str>>,
		take_while1::<_, &str, VerboseError<&str>>(|ch| ch != '\n')
	))(input).map_err(crate::MAP_ERR);
	let var = res?.1.to_owned();
	Ok(Variable {
		identifier: var.2.trim().to_string(),
		value: var.5.trim().to_string()
	})
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variable {
	pub identifier: String,
	pub value: String
}