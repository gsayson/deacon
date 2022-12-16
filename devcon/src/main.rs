mod commands;
mod integrations;
mod util;
mod env;

use std::io::Read;
use std::path::Path;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use ansi_term::Colour::*;
use chrono::{NaiveDateTime, TimeZone, Utc};
use duct::ReaderHandle;
use guess_host_triple::guess_host_triple;
use rustyline::*;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use crate::commands::resolve_function;
use crate::env::execute_process;
use crate::integrations::rust::{get_crate_manifest, is_cargo_workspace};
use crate::util::print_prompt;

fn main() -> Result<()> {
    //return Err(ReadlineError::Io(io::Error::last_os_error()));
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
    println!("{}", RGB(255, 165, 0).bold().paint(format!("Current PID: {}", std::process::id())));
    let mut rl = Editor::<()>::with_config(
        Config::builder()
            .completion_type(CompletionType::List)
            .build()
    )?;
    rl.set_color_mode(ColorMode::Enabled);
    let history_path = Path::new(".devcon-history.txt");
    if rl.load_history(history_path).is_ok() {
        let md = history_path.metadata().expect("able to get metadata");
        match md.modified() {
            Ok(time) => {
                fn system_time_to_date_time(t: SystemTime) -> NaiveDateTime {
                    let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
                        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
                        Err(e) => {
                            let dur = e.duration();
                            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
                            if nsec == 0 {
                                (-sec, 0)
                            } else {
                                (-sec - 1, 1_000_000_000 - nsec)
                            }
                        },
                    };
                    Utc.timestamp_opt(sec, nsec).unwrap()
                        .naive_local()
                }
                println!("Restored previous session on {}.", Blue.bold().paint(system_time_to_date_time(time).format("%d %B %Y %I:%M %p").to_string()));
            }
            Err(_) => {
                println!("Restored previous session.");
            }
        }
    }
    println!("For help, type `help` and hit enter.\n");
    loop {
        util::print_prompt();
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if is_blank(&line) {
                    println!();
                    continue;
                } else if !line.starts_with("exit") {
                    rl.add_history_entry(line);
                    if line.starts_with("clear") {
                        print!("\x1B[2J\x1B[1;1H");
                    } else if !resolve_function(&line) {
                        // execute a process
                        if let Some((command, mut child)) = execute_process(line) {
                            // let stdin = child.stdin.take().unwrap();
                            // let stdout = child.stdout.take().unwrap();
                            // let (so, ra) = mpsc::channel::<bool>();
                            // println!("{}", stdout.take())
                            // currently, we only print to console
                            // piping to a file tbd
                            while let Ok(None) = child.try_wait() {
                                // pass
                            }
                            println!();
                        }
                    } else {
                        println!();
                    }
                } else {
                    break
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!();
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
