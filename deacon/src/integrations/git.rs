use std::path::Path;
use git2::{BranchType, Repository};
use nerd_fonts::NerdFonts;

pub fn is_git_project(dir: impl AsRef<Path>) -> bool {
	Repository::open(dir.as_ref()).is_ok()
}

pub fn get_integration(dir: impl AsRef<Path>) -> Option<String> {
	let repo = Repository::open(dir.as_ref()).unwrap();
	// let t = match repo.revwalk() {
	// 	Ok(mut ok) => {
	// 		let i = ok.next();
	// 		if let Some(i) = i {
	// 			match i {
	// 				Ok(oid) => {
	// 					let commit = repo.find_commit(oid).unwrap(); // this will never panic
	// 					Some(format!("[{}: {}]", commit.author(), oid.to_string().substring(0, 6)))
	// 				}
	// 				Err(_) => {
	// 					None
	// 				}
	// 			}
	// 		} else {
	// 			None
	// 		}
	// 	}
	// 	Err(_) => {
	// 		None
	// 	}
	// };
	// t
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