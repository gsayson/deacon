use std::path::{Path, PathBuf};
use git2::{BranchType, Repository};
use nerd_fonts::NerdFonts;

pub fn get_nearest_git_repository(dir: impl AsRef<Path>) -> Option<PathBuf> {
	let dir = dir.as_ref();
	if Repository::open(dir).is_ok() {
		Some(dir.to_path_buf())
	} else {
		if let Some(parent) = dir.parent() {
			get_nearest_git_repository(parent)
		} else {
			None
		}
	}
}

pub fn get_integration(dir: impl AsRef<Path>) -> Option<String> {
	let dir = dir.as_ref();
	let repo = Repository::open(dir).unwrap();
	let binding = repo.branches(Some(BranchType::Local)).ok()?.next()?.ok()?;
	let br = binding.0.name().ok()??;
	Some(
		format!(
			"[{}branch \"{}\"]",
			NerdFonts { nf: NerdFonts::load() }.get("nf-dev-git_branch").map_or("".to_string(), |f| f.to_string() + " "),
			br
		)
	)
}