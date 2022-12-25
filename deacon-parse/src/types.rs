//! Deacon types.

use std::hint::unreachable_unchecked;
use nom::character::complete::{alpha1, char, multispace0};
use nom::error::VerboseError;
use nom::multi::separated_list0;
use nom::sequence::delimited;
use crate::types::DeaconType::Tuple;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum DeaconType {
	/// A UTF8-encoded growable string.
	String,
	/// A 64-bit signed integer.
	Int,
	/// A Boolean value.
	Bool,
	/// The `null` value.
	Null,
	/// A tuple of types.
	Tuple(Vec<DeaconType>)
}

impl<'a> TryFrom<&'a str> for DeaconType {
	type Error = VerboseError<&'a str>;
	fn try_from(value: &'a str) -> Result<DeaconType, Self::Error> {
		match value.as_ref() {
			"string" => Ok(DeaconType::String),
			"int" => Ok(DeaconType::Int),
			"bool" => Ok(DeaconType::Bool),
			"null" => Ok(DeaconType::Null),
			tuple => {
				let tuple = tuple.trim();
				let res = delimited(
					char::<&str, VerboseError<&str>>('('),
					separated_list0(char::<&str, VerboseError<&str>>(','),  nom::sequence::tuple((multispace0, alpha1, multispace0))),
					char::<&str, VerboseError<&str>>(')')
				)(tuple).map_err(|f| {
					match f {
						nom::Err::Incomplete(_) => unsafe { unreachable_unchecked() },
						nom::Err::Error(e) | nom::Err::Failure(e) => e
					}
				});
				if let Ok(f) = res {
					let types = f.1.into_iter().map(|f| f.1).collect::<Vec<&str>>();
					let mut deacon_types: Vec<DeaconType> = vec![];
					for t in types {
						let t = t.trim();
						deacon_types.push(DeaconType::try_from(t)?);
					}
					Ok(Tuple(deacon_types))
				} else {
					Err(res.err().unwrap())
				}
			}
		}
	}
}