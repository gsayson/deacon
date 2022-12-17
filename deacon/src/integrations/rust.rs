//! Integration for the Rust programming language.

use std::fs;
use std::path::Path;
use cargo_toml::Manifest;

pub fn is_rust_project(dir: impl AsRef<Path>) -> bool {
	let mut dir = dir.as_ref().to_path_buf();
	let dir_2 = dir.clone();
	dir.push("Cargo.toml");
	dir.exists() || match dir_2.parent() {
		None => false,
		Some(parent) => is_rust_project(parent)
	}
}

pub fn is_cargo_workspace(dir: impl AsRef<Path>) -> bool {
	let mut dir = dir.as_ref().to_path_buf();
	dir.push("Cargo.toml");
	match fs::read_to_string(dir) {
		Ok(contents) => {
			contents.split("\n").any(|x| x.trim().starts_with("[workspace]"))
		}
		Err(_) => {
			false
		}
	}
}

pub fn get_crate_manifest(dir: impl AsRef<Path>) -> Option<Manifest> {
	let mut dir = dir.as_ref().to_path_buf();
	let dir_2 = dir.clone();
	dir.push("Cargo.toml");
	let res = Manifest::from_path(dir);
	match res {
		Ok(manifest) => {
			Some(manifest)
		}
		Err(_) => {
			if let Some(parent) = dir_2.parent() {
				get_crate_manifest(parent)
			} else {
				None
			}
		}
	}
}