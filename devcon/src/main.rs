mod commands;
mod integrations;

use ansi_term::Colour::*;
use guess_host_triple::guess_host_triple;
use rustyline::*;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use crate::commands::resolve_function;


fn main() -> Result<()> {
    #[cfg(windows)]
    ansi_term::enable_ansi_support().unwrap();
    println!("{} [{} {} on {}]",
             Yellow.bold().paint("DevCon"),
             Cyan.paint(env!("CARGO_PKG_VERSION")),
             {
                if !env!("CARGO_PKG_VERSION").starts_with("0.") {
                    Green.bold().paint("stable")
                } else {
                    Red.bold().paint("unstable")
                }
            },
            Cyan.bold().paint(guess_host_triple().unwrap_or("unknown"))
    );
    let mut rl = Editor::<()>::new()?;
    rl.set_color_mode(ColorMode::Enabled);
    if rl.load_history(".devcon-history.txt").is_ok() {
        println!("Imported previous command history.");
    }
    println!("For help, type `help` and hit enter.");
    loop {
        println!(
            "\n{}: {} ",
            Green.bold().paint(format!("{}@{}", whoami::username(), whoami::hostname())),
            Blue.bold().paint(std::env::current_dir().unwrap().to_string_lossy().trim_end())
        );
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                if is_blank(&line) {
                    continue;
                }
                rl.add_history_entry(line.as_str());
                if line.starts_with("exit") {
                    return Ok(());
                }
                if !resolve_function(line) {
                    println!("unknown internal function");
                }
            },
            Err(ReadlineError::Interrupted) => {
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history(".devcon-history.txt")
}

pub fn is_blank(input: impl AsRef<str>) -> bool {
    input.as_ref().split_whitespace().next().is_none()
}