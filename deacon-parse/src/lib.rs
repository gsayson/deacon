use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::char;
use nom::error::ParseError;
use nom::{AsChar, InputTakeAtPosition, IResult};
use nom::multi::many1;
use nom::sequence::delimited;

/// Parses environment variables. If there is no environmental variable to substitute, this function will return [`None`].
pub fn parse_env_vars(input: &str) -> Option<Vec<&str>> {

	let res = many1(delimited(
		char('$'),
		alpha_underscore_0::<&str, ()>,
		char('$')
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

fn alpha_underscore_0<T: InputTakeAtPosition, E: ParseError<T>>(input: T) -> IResult<T, T, E> where <T as InputTakeAtPosition>::Item: AsChar {
	input.split_at_position_complete(|item| !({
		let ch = item.as_char();
		ch.is_alpha() || ch == '_'
	}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_env_var() {
	    assert_eq!(parse_env_vars("$"), None);
	    assert_eq!(parse_env_vars("$$"), Some(vec![""]));
	    assert_eq!(parse_env_vars("$$$"), Some(vec![""]));
	    assert_eq!(parse_env_vars("$hi$$"), Some(vec!["hi"]));
	    assert_eq!(parse_env_vars("$__$$"), Some(vec!["__"]));
	    assert_eq!(parse_env_vars("$hi$$$"), Some(vec!["hi", ""]));
	    assert_eq!(parse_env_vars("$gamma_delta$"), Some(vec!["gamma_delta"]));
	    assert_eq!(parse_env_vars("$$$$"), Some(vec!["", ""]));
	    assert_eq!(parse_env_vars("$foo$"), Some(vec!["foo"]));
	    assert_eq!(parse_env_vars("$space$$bar$"), Some(vec!["space", "bar"]));
    }

}