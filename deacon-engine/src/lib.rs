#![feature(exact_size_is_empty)]

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use ariadne::{Label, ReportBuilder, Source};
use nom::error::{VerboseError, VerboseErrorKind};
use deacon_parse::function::*;
use deacon_parse::types::DeaconType;
use deacon_parse::variable::*;

// /// Runs the given script. Callers **must** lint the script for errors
// /// before running; not doing so can cause side effects for
// /// the current environment (if the current environment is **`using`** the
// /// given script).
// /// ```no-run
// /// # equivalent to `run_script("hello.dc", false);`
// /// # because it starts a subshell child process
// /// # `export` will be no-op
// /// deacon --run ./hello.dc
// ///
// /// # equivalent to `run_script("hello.dc", true);`
// /// # because it runs the script in the current process
// /// # export will actually have an effect here
// /// using ./hello.dc
// /// ```
// pub fn run_script(file: impl AsRef<Path>, using: bool) -> io::Result<()> {
//     let mut file = File::open(file)?;
//     let mut input = String::new();
//     file.read_to_string(&mut input)?;
//     let input = input.as_str();
//     deacon_parse::function::parse_func_declaration()
// }

pub fn lint_script(file: impl AsRef<Path>) -> Option<()> {
    let path = file.as_ref();
    let mut file = File::open(path).ok()?;
    let mut input = String::new();
    file.read_to_string(&mut input).ok()?;
    let full_input = input.clone();
    let mut lines = input.lines().map(|f| f.to_string()).collect::<Vec<String>>().into_iter(); // gain access to is_empty
    let mut function_vec = vec![];
    let mut input = "".to_string();
    let mut to_reset = true;
    while !lines.is_empty() {
        if to_reset {
            input = lines.next().unwrap();
        }
        'func_parse: loop {
            match parse_func_declaration(&input) {
                Ok((func, remainder)) => {
                    function_vec.push(func);
                    input = remainder.to_string();
                    to_reset = true;
                    break 'func_parse;
                }
                Err(err) => {
                    match err {
                        nom::Err::Incomplete(_) => {
                            // try again.
                            input = input + &*lines.next()?;
                            to_reset = false;
                        }
                        nom::Err::Error(e) => {
                            for (affected, kind) in e.errors {
                                match kind {
                                    VerboseErrorKind::Context(_) => {}
                                    VerboseErrorKind::Char(expected) => {
                                        let pb = path.to_path_buf();
                                        let pb2 = pb.file_name().unwrap().to_string_lossy();
                                        let name = pb2.as_ref();
                                        println!("{}", affected);
                                        let mut b = ariadne::Report::build(ariadne::ReportKind::Error, name, full_input.find(affected).unwrap())
                                            .with_message(format!("Expected '{}'.", expected.escape_default()))
                                            .with_label(Label::new((name, 4..7)));
                                        if expected.escape_default().to_string() == "\\n" {
                                            b.set_note("The '{' character must be followed by a newline, and the '}' character must be after another newline.");
                                        }
                                        let _ = b.with_code(0) // parsing error is E[0] b
                                            .finish()
                                            .eprint((name, Source::from(&input)));
                                    }
                                    VerboseErrorKind::Nom(error) => {}
                                }
                            }
                            to_reset = true;
                            break 'func_parse;
                        }
                        nom::Err::Failure(e) => {
                            // CONFIRMED: we're on correct branch!
                            eprintln!("{}", nom::error::convert_error(input.as_str(), e));
                            to_reset = true;
                            break 'func_parse;
                        }
                    }
                }
            }
        }
    }
    Some(())
}

/// The `Either` type.
pub enum Either<A, B> {
    A(A),
    B(B),
}