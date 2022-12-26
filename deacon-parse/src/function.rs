//! Function declaration handlers and tables.

use std::hint::unreachable_unchecked;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{alpha1, char, multispace0};
use nom::Err::Incomplete;
use nom::error::{ErrorKind, ParseError, VerboseError};
use nom::{IResult, Needed};
use nom::multi::separated_list0;
use nom::sequence::{delimited, terminated, tuple};
use crate::alpha_underscore_1;

/// Parses a function. The syntax for a function is:
/// ```sh
/// 'export'? 'func' IDENTIFIER '(' (TYPE IDENTIFIER)* ')' '{'
///     STATEMENT*
/// '}'
/// ```
/// So, an example function which simply runs the `echo` command can be defined like:
/// ```sh
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

	// parsing header start
	let mut input = input;
	let export_tag = tag::<_, &str, ()>("export")(input);
	let is_exported = export_tag.is_ok();
	if is_exported {
		input = export_tag.unwrap().0;
	}
	let input = input.trim();
	let header = tag::<_, &str, VerboseError<&str>>("func")(input)
		.map_err(|f| match f {
			Incomplete(_) => unsafe { unreachable_unchecked() }
			nom::Err::Error(_) | nom::Err::Failure(_) => {
				Incomplete(Needed::Unknown) // returning incomplete gives a telltale sign of a func decl.; Needed::Unknown is going to be our magic value.
			}
		})?.0.trim();
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
	let (remainder, statements): (&str, &str) = delimited(
		char::<&str, VerboseError<&str>>('{'),
		take_until_unbalanced('{', '}'), // doesn't support nested {} yet; use parse_hyperlinks::take_until_unbalanced
		char::<&str, VerboseError<&str>>('}')
	)(code_block)?;
	let statements = statements.trim().lines();
	let statements = statements.into_iter().map(|f| f.trim().to_string()).collect::<Vec<String>>();
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
/// ```sh
/// COMMAND ARGUMENT* | FUNCTION ARGUMENT*
/// ```
/// Calls are the building blocks of Deacon. When you execute any command,
/// you are executing a call. For example (note that `$` is the default prompt and therefore it is present here),
/// ```sh
/// $ help
/// ```
///
/// Calls, other than being the execution of commands, can also be
/// function calls. For example,
/// ```sh
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

//                   MIT LICENSE:
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//
// take_until_unbalanced: Copyright (C) the maintainer of `parse_hyperlinks`
fn take_until_unbalanced(
	opening_bracket: char,
	closing_bracket: char,
) -> impl Fn(&str) -> IResult<&str, &str, VerboseError<&str>> {
	move |i: &str| {
		let mut index = 0;
		let mut bracket_counter = 0;
		while let Some(n) = &i[index..].find(&[opening_bracket, closing_bracket, '\\'][..]) {
			index += n;
			let mut it = i[index..].chars();
			match it.next().unwrap_or_default() {
				c if c == '\\' => {
					// Skip the escape char `\`.
					index += '\\'.len_utf8();
					// Skip also the following char.
					let c = it.next().unwrap_or_default();
					index += c.len_utf8();
				}
				c if c == opening_bracket => {
					bracket_counter += 1;
					index += opening_bracket.len_utf8();
				}
				c if c == closing_bracket => {
					// Closing bracket.
					bracket_counter -= 1;
					index += closing_bracket.len_utf8();
				}
				// Can not happen.
				_ => unreachable!(),
			};
			// We found the unmatched closing bracket.
			if bracket_counter == -1 {
				// We do not consume it.
				index -= closing_bracket.len_utf8();
				return Ok((&i[index..], &i[0..index]));
			};
		}

		if bracket_counter == 0 {
			Ok(("", i))
		} else {
			Err(nom::Err::Error(VerboseError::from_error_kind(i, ErrorKind::TakeUntil)))
		}
	}
}