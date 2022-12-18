use std::path::Path;
use git2::{BranchType, Repository};
use nerd_fonts::NerdFonts;

pub fn is_git_project(dir: impl AsRef<Path>) -> bool {
	let dir = dir.as_ref();
	Repository::open(dir).is_ok()
	|| {
		if let Some(parent) = dir.parent() {
			is_git_project(dir)
		} else {
			false
		}
	}
}

pub fn get_integration(dir: impl AsRef<Path>) -> Option<String> {
	let repo = Repository::open(dir.as_ref()).unwrap();
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