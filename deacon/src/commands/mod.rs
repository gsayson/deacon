//! Internal commands.

use std::env;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use ansi_term::Colour::*;
use byte_unit::Byte;
use chrono::{Local, NaiveDateTime};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use crate::util::print_help;

// input is guaranteed to NOT be blank.
pub fn resolve_function(input: impl AsRef<str>) -> bool {
	let input = input.as_ref();
	if input.trim().starts_with("!") {
		// execute literally the given process and its args
		return false;
	}
	match input.split_whitespace().next().unwrap() {
		"cd" => change_dir(input),
		"devconinfo" => print_devcon_info(),
		"help" => print_help(),
		"ls" => list_dir(input),
		&_ => {
			return false;
		}
	}
	return true
}

pub fn change_dir(input: impl AsRef<str>) {
	let input = input.as_ref();
	let path = input.split_whitespace().skip(1).next();
	if let Some(path) = path {
		match env::set_current_dir(path) {
			Ok(_) => {}
			Err(err) => {
				eprintln!("{}", Red.paint(format!("Failed to change directory: {}", err.to_string())));
			}
		}
	} else {
		eprintln!("{}", Red.paint("Please provide a path."));
	}
}

pub fn print_devcon_info() {
	println!("{} {} {}",
	         Yellow.bold().paint("DevCon"),
	         Cyan.paint(env!("CARGO_PKG_VERSION")),
	         {
		         if !env!("CARGO_PKG_VERSION").starts_with("0.") {
			         Green.bold().paint("stable")
		         } else {
			         Red.bold().paint("unstable")
		         }
	         },
	);
	println!("debug build: {}", crate::util::colorize_bool(cfg!(debug_assertions)));
}

pub fn list_dir(input: impl AsRef<str>) {
	let mut table = Table::new();
	table.load_preset(UTF8_FULL);
	let mut input = input.as_ref().split_whitespace();
	input.next().unwrap();
	let mut dir = env::current_dir();
	if let Some(dir_inner) = input.next() {
		dir = Ok(PathBuf::from(dir_inner));
	}
	match dir {
		Ok(dir) => {
			println!("Listing for {}", dir.display());
			table.set_header(vec![
				"File",
				"Type",
				"Created",
				"Modified",
				"Size"
			]);
			match dir.read_dir() {
				Ok(read_dir) => {
					for entry in read_dir {
						if let Ok(entry) = entry && entry.path().metadata().is_ok() {
							let metadata_available = entry.path().metadata().is_ok();
							let x = {
								match entry.path().metadata() {
									Ok(md) => {
										Byte::from_bytes(md.len() as u128)
											.get_appropriate_unit(false)
											.to_string()
									}
									Err(_) => {
										String::from("Unavailable")
									}
								}
							};
							table.add_row(vec![
								entry.path().canonicalize().unwrap().file_name().unwrap().to_string_lossy().as_ref(),
								if entry.path().is_file() {
									"File"
								} else if entry.path().is_dir() {
									"Directory"
								} else if entry.path().is_symlink() {
									"Symlink"
								} else {
									"Unknown"
								},
								{
									if metadata_available {
										let dt = NaiveDateTime::default();
										let dur = entry.path().metadata().unwrap().created().unwrap()
											.duration_since(UNIX_EPOCH)
											.unwrap();
										dt.checked_add_signed(
											chrono::Duration::from_std(dur).unwrap()
										).unwrap().and_local_timezone(Local).latest().unwrap().format("%d %B %Y %I:%M %p").to_string()
									} else {
										"Unavailable".to_string()
									}
								}.as_str(),
								{
									if metadata_available {
										let dt = NaiveDateTime::default();
										let dur = entry.path().metadata().unwrap().modified().unwrap()
											.duration_since(UNIX_EPOCH)
											.unwrap();
										dt.checked_add_signed(
											chrono::Duration::from_std(dur).unwrap()
										).unwrap().and_local_timezone(Local).latest().unwrap().format("%d %B %Y %I:%M %p").to_string()
									} else {
										"Unavailable".to_string()
									}
								}.as_str(),
								if entry.path().is_file() {
									x.as_str()
								} else {
									"N/A"
								}
							]);
						}
					}
					println!("{}", table);
				}
				Err(err) => {
					eprintln!("{}", Red.paint(format!("Failed to read directory: {}", err.to_string())));
				}
			}
		}
		Err(err) => {
			eprintln!("{}", Red.paint(format!("Failed to list directory: {}", err.to_string())));
		}
	}
}

#[test]
fn s() {
    println!("{}", PathBuf::from("D:\\Intrinsic Native Virtual Machine.docx").canonicalize().unwrap().file_name().unwrap().to_string_lossy().as_ref());
}