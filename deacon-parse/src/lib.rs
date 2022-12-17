// use nom::branch::alt;
// use nom::bytes::complete::take_while;
// use nom::character::complete::{alpha0, char};
// use nom::sequence::delimited;
//
// pub fn parse_env_vars(input: impl ToString) {
//     let input = input.to_string();
//     delimited(
//         char('$'),
//         alpha0,
//         char('$')
//     )(input).expect("panic message");
// }