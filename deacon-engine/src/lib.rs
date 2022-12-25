#![feature(exact_size_is_empty)]

use std::fs::File;
use std::io::Read;
use std::path::Path;
use ariadne::{Label, Source};
use nom::error::VerboseErrorKind;
use deacon_parse::function::*;

// /// Runs the given script. Callers **must** lint the script for errors
// /// before running; not doing so can cause side effects for
// /// the current environment (if the current environment is **`using`** the
// /// given script).
// /// ```sh
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
    let mut function_vec = vec![];
    loop {
        if input.trim().is_empty() {
            break
        }
        match parse_func_declaration(&input) {
            Ok((func, remainder)) => {
                println!("{:?}", func);
                function_vec.push(func);
                let remainder = remainder.trim();
                input = remainder.to_string();
                continue
            }
            Err(err) => {
                match err {
                    nom::Err::Incomplete(_) => {
                        println!("Incomplete!");
                        break
                    }
                    nom::Err::Error(e) => {
                        for(affected, kind) in e.errors {
                            match kind {
                                VerboseErrorKind::Context(ctx) => {
                                    println!("Context: {}", ctx);
                                }
                                VerboseErrorKind::Char(expected) => {
                                    let pb = path.to_path_buf();
                                    let pb2 = pb.file_name().unwrap().to_string_lossy();
                                    let name = pb2.as_ref();
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
                                VerboseErrorKind::Nom(error) => {
                                    println!("ErrorKind: {:?}", error);
                                }
                            }
                        }
                        break
                    }
                    nom::Err::Failure(e) => {
                        // CONFIRMED: we're on correct branch!
                        eprintln!("Oh no!\n{}", nom::error::convert_error(input.as_str(), e));
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