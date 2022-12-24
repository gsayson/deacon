//! Function declaration handlers and tables.

use std::hint::unreachable_unchecked;
use nom::bytes::complete::{tag, take_till, take_while};
use nom::character::complete::{alpha1, char, multispace0, newline};
use nom::combinator::cut;
use nom::error::VerboseError;
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, terminated, tuple};
use crate::alpha_underscore_1;

/// Parses a function. The syntax for a function is:
/// ```no-run
/// 'export'? 'func' IDENTIFIER '(' (TYPE IDENTIFIER)* ')' '{'
///     STATEMENT*
/// '}'
/// ```
/// So, an example function which simply runs the `echo` command can be defined like:
/// ```no-run
/// func echo(parameter: string) {
///     echo $parameter
/// }
/// ```
/// This substitutes the given `parameter` of the `echo` *function* into the argument placeholder of the `echo` *command*.
/// Note that functions' names cannot be the names of reserved keywords, such as `func`, `using`, or the name of any data type.
///
/// Exporting a function allows the **current** shell environment to run that function.
///
/// For more details on statements, see [`parse_call`].
///
/// Note that while `func` is a reserved keyword, if there is an executable literally named `func` one can run it
/// using the exclamation-escape syntax, in this case `!func`.
///
/// This function, compared to other functions, returns a layer of [metadata](nom::Err) since this
/// parser uses **streaming** parsers (since it has to handle newlines).
pub fn parse_func_declaration(input: &str) -> Result<(Function, &str), nom::Err<VerboseError<&str>>> {

	use nom::bytes::streaming::{tag, take_till, take_while};
	use nom::character::streaming::{alpha1, char, multispace0, newline};

	// parsing header start
	let mut input = input;
	let export_tag = tag::<_, &str, ()>("export")(input);
	let is_exported = export_tag.is_ok();
	if is_exported {
		input = export_tag.unwrap().0;
	}
	let input = input.trim();
	let header = cut(tag::<_, &str, VerboseError<&str>>("func"))(input)?.0.trim();
	let (args, name) = terminated(alpha_underscore_1::<&str, VerboseError<&str>>, tag::<_, &str, VerboseError<&str>>("("))(header)?;
	let (code_block, args) = terminated(
		separated_list0(
			tuple((char::<&str, VerboseError<&str>>(','), multispace0::<&str, VerboseError<&str>>)),
			tuple((alpha_underscore_1::<&str, VerboseError<&str>>, multispace0::<&str, VerboseError<&str>>, char::<&str, VerboseError<&str>>(':'), multispace0::<&str, VerboseError<&str>>, alpha1::<&str, VerboseError<&str>>))
		),
		tag(")") // discard
	)(args)?;
	// parsing header end
	// parsing block starts
	let code_block = code_block.trim();
	let (remainder, statements): (&str, Vec<(_, &str, _, _)>) = delimited(
		tuple((char::<&str, VerboseError<&str>>('{'), newline::<&str, VerboseError<&str>>)),
		many0(tuple((
			multispace0::<&str, VerboseError<&str>>,
			take_till(|f| f == '\n'),
			newline::<&str, VerboseError<&str>>,
			multispace0::<&str, VerboseError<&str>>
			))),
		char::<&str, VerboseError<&str>>('}')
	)(code_block)?;
	let statements = statements.into_iter().map(|f| f.1.trim().to_string()).collect::<Vec<String>>();
	// parsing block end
	Ok((Function {
			name: name.to_string(),
			args: {
				let args = args;
				args.into_iter()
					.map(|f| {
						FormalArg {
							identifier: f.0.to_string(),
							r#type: f.4.to_string(),
						}
					})
					.collect::<Vec<FormalArg>>()
			},
			body: statements,
		},
	remainder))
}

/// Parses a call. The syntax for a call is:
/// ```no-run
/// COMMAND ARGUMENT* | FUNCTION ARGUMENT*
/// ```
/// Calls are the building blocks of Deacon. When you execute any command,
/// you are executing a call. For example (note that `$` is the default prompt and therefore it is present here),
/// ```no-run
/// $ help
/// ```
///
/// Calls, other than being the execution of commands, can also be
/// function calls. For example,
/// ```no-run
/// $ echo "string"
/// ```
/// makes a call to the `echo` function (defined in the documentation of the [`parse_fn`](self::parse_func_declaration) function).
pub fn parse_call(input: &str) -> Result<Call, VerboseError<&str>> {
	match take_while::<_, &str, VerboseError<&str>>(|i: char| !i.is_ascii_whitespace())(input) {
		Ok(output) => {
			Ok(Call {
				name: String::from(output.1),
				args: {
					let s = output.0.split_ascii_whitespace();
					let vec = s.collect::<Vec<&str>>();
					vec.into_iter().map(|f| f.to_string())
						.collect::<Vec<String>>()
				},
			})
		}
		Err(e) => {
			match e {
				nom::Err::Incomplete(_) => unsafe {
					// SAFETY: complete (non-streaming) parsers never return nom::Err::Incomplete.
					unreachable_unchecked()
				}
				nom::Err::Error(e) | nom::Err::Failure(e) => {
					Err(e)
				}
			}
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
	pub name: String,
	pub args: Vec<String>
}

#[derive(Debug, PartialEq, Clone)]
pub struct FormalArg {
	identifier: String,
	r#type: String
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
	pub name: String,
	pub args: Vec<FormalArg>,
	pub body: Vec<String>
}