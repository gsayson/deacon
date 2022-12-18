//! Internal commands.

use std::env;
use std::time::UNIX_EPOCH;
use ansi_term::Colour::*;
use byte_unit::Byte;
use chrono::{Local, NaiveDateTime, Offset, TimeZone};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use crate::util::print_help;

// input is guaranteed to NOT be blank.
pub fn resolve_function(input: impl AsRef<str>) -> bool {
	let input = input.as_ref();
	match input.split_whitespace().next().unwrap() {
		"cd" => change_dir(input),
		"devconinfo" => print_devcon_info(),
		"help" => print_help(),
		"dir" => list_dir(),
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

pub fn list_dir() {
	let mut table = Table::new();
	table.load_preset(UTF8_FULL);
	let dir = env::current_dir();
	match dir {
		Ok(dir) => {
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
						if let Ok(entry) = entry {
							let x = Byte::from_bytes(entry.path().metadata().unwrap().len() as u128)
								.get_appropriate_unit(false)
								.to_string();
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
									let dt = NaiveDateTime::default();
									let dur = entry.path().metadata().unwrap().created().unwrap()
										.duration_since(UNIX_EPOCH)
										.unwrap();
									dt.checked_add_signed(
										chrono::Duration::from_std(dur).unwrap()
									).unwrap().and_local_timezone(Local).latest().unwrap().format("%d %B %Y %I:%M %p").to_string()
								}.as_str(),
								{
									let dt = NaiveDateTime::default();
									let dur = entry.path().metadata().unwrap().modified().unwrap()
										.duration_since(UNIX_EPOCH)
										.unwrap();
									dt.checked_add_signed(
										chrono::Duration::from_std(dur).unwrap()
									).unwrap().and_local_timezone(Local).latest().unwrap().format("%d %B %Y %I:%M %p").to_string()
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