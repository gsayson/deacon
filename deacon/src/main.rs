#![feature(let_chains)]
#![feature(exact_size_is_empty)]
#![feature(iter_intersperse)]

mod commands;
mod integrations;
mod util;
mod env;

use std::borrow::Cow;
use std::borrow::Cow::{Borrowed, Owned};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use ansi_term::Colour::*;
use chrono::*;
use guess_host_triple::guess_host_triple;
use rustyline::*;
use rustyline::completion::FilenameCompleter;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::validate::MatchingBracketValidator;
use rustyline_derive::*;
use crate::commands::resolve_function;
use crate::env::execute_process;
use crate::util::print_prompt;

const ASCII_LOGO: &str =
r#"    ____
   / __ \___  ____ __________  ____
  / / / / _ \/ __ `/ ___/ __ \/ __ \
 / /_/ /  __/ /_/ / /__/ /_/ / / / /
/_____/\___/\__,_/\___/\____/_/ /_/"#;

#[derive(Helper, Completer, Hinter, Validator)]
struct MyHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator, // replace soon with deacon engine validator
    colored_prompt: String,
}

impl Highlighter for MyHelper {
    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        Owned(line.to_string())
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(RGB(55, 55, 55).paint(hint).to_string())
    }

    fn highlight_char(&self, _: &str, _: usize) -> bool {
        false
    }
}

fn main() -> Result<()> {
    #[cfg(windows)]
    ansi_term::enable_ansi_support().unwrap();
    println!("{}\n", Yellow.bold().paint(ASCII_LOGO));
    println!("{} [{} {} on {}]",
             Yellow.bold().paint("Deacon Shell"),
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
    println!("{}", RGB(255, 165, 0).bold().paint(format!("Current PID: {}{}", std::process::id(), {
        if Local::now().month() == 12 {
            " | Merry Christmas!"
        } else if Local::now().month() == 10 {
            " | Happy Halloween!"
        } else {
            ""
        }
    })));

    std::panic::set_hook(Box::new(|panic_info| {
        use ariadne::*;
        let location = panic_info.location().unwrap(); // current version always returns `Some`
        let x = *panic_info.payload().downcast_ref::<&str>().unwrap_or(&"What just happened??");
        Report::build(ReportKind::Error, (), 0)
            .with_code("panic")
            .with_message("Please file an issue on our GitHub and report this bug!")
            .with_label(Label::new(0..x.len()).with_message("Sneaky little bug!"))
            .with_help(format!("Looks like it happened in '{}', {}:{}", location.file().replace("\\", "/"), location.line(), location.column()))
            .with_note("You can find Deacon at https://github.com/gsayson/deacon/")
            .finish()
            .eprint(Source::from(x))
            .unwrap_or(());
    }));

    let mut rl = Editor::<MyHelper>::with_config(
        Config::builder()
            .completion_type(CompletionType::Circular)
            .history_ignore_space(true)
            .build()
    )?;
    rl.set_color_mode(ColorMode::Enabled);
    rl.set_helper(Some(MyHelper {
        completer: FilenameCompleter::new(),
        validator: MatchingBracketValidator::new(),
        colored_prompt: "$ ".to_string(),
    }));
    let history_path = Path::new(".devcon-history.txt");
    if rl.load_history(history_path).is_ok() {
        let md = history_path.metadata().expect("able to get metadata");
        match md.modified() {
            Ok(time) => {
                fn system_time_to_date_time(t: SystemTime) -> DateTime<Local> {
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
                    NaiveDateTime::from_timestamp_opt(sec, nsec).unwrap().and_local_timezone(Local).earliest().unwrap()
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
        print_prompt();
        let readline = rl.readline("  ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if is_blank(&line) {
                    println!();
                    continue;
                } else {
                    let line = env::substitute_env_var(line);
                    let line = line.split_whitespace()
                        .map(|f| if f.trim().starts_with("\"") || f.trim().starts_with("\'") {
                            f.to_string()
                        } else {
                            f.replace("~", &*dirs::home_dir().map(|f| f.to_string_lossy().to_string()).unwrap_or(String::from("~")))
                        }).intersperse(" ".parse().unwrap())
                        .collect::<String>();
                    let line = &line;
                    if !line.starts_with("exit") {
                        rl.add_history_entry(line);
                        if line.starts_with("clear") {
                            print!("\x1B[2J\x1B[1;1H");
                        } else if !resolve_function(&line) {
                            // execute a process
                            if let Some((_command, mut child)) = execute_process(line) {
                                // let pid = child.id();
                                while let Ok(None) = child.try_wait() {
                                    // if rx.try_recv().is_ok() {
                                    //     #[cfg(windows)]
                                    //     unsafe {
                                    //         use windows::Win32::System::Console::*;
                                    //         AttachConsole(pid);
                                    //         GenerateConsoleCtrlEvent(CTRL_C_EVENT, pid);
                                    //     }
                                    // }
                                }
                            }
                            println!();
                        } else {
                            println!();
                        }
                    } else {
                        break
                    }
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
