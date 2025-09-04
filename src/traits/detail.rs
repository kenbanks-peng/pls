use crate::config::EntryConst;
use crate::enums::{DetailField, Typ};
use crate::ext::Ctime;
use crate::models::{Node, OwnerMan, Perm};
use crate::PLS;
use git2::{Repository, Status};
use log::warn;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::time::SystemTime;
use time::{format_description, OffsetDateTime, UtcOffset};

pub trait Detail {
	fn size_val(&self) -> Option<u64>;
	fn blocks_val(&self) -> Option<u64>;
	fn time_val(&self, field: DetailField) -> Option<SystemTime>;
	fn user_val(&self, owner_man: &mut OwnerMan) -> Option<String>;
	fn group_val(&self, owner_man: &mut OwnerMan) -> Option<String>;

	fn dev(&self, entry_const: &EntryConst) -> Option<String>;
	fn ino(&self, entry_const: &EntryConst) -> Option<String>;
	fn nlink(&self, entry_const: &EntryConst) -> Option<String>;
	fn perm(&self, entry_const: &EntryConst) -> Option<String>;
	fn oct(&self, entry_const: &EntryConst) -> Option<String>;
	fn user(&self, owner_man: &mut OwnerMan, entry_const: &EntryConst) -> Option<String>;
	fn uid(&self, owner_man: &mut OwnerMan, entry_const: &EntryConst) -> Option<String>;
	fn group(&self, owner_man: &mut OwnerMan, entry_const: &EntryConst) -> Option<String>;
	fn gid(&self, owner_man: &mut OwnerMan, entry_const: &EntryConst) -> Option<String>;
	fn size(&self, entry_const: &EntryConst) -> Option<String>;
	fn blocks(&self, entry_const: &EntryConst) -> Option<String>;
	fn time(&self, field: DetailField, entry_const: &EntryConst) -> Option<String>;
	fn git(&self, entry_const: &EntryConst) -> Option<String>;
}

impl Detail for Node<'_> {
	// ===========
	// Sort fields
	// ===========

	/// Compute the size of the node, returning `None` for directories.
	fn size_val(&self) -> Option<u64> {
		self.meta_ok()
			.filter(|_| self.typ != Typ::Dir)
			.map(|meta| meta.len())
	}

	/// Compute the block count for the node, returning `None` for directories.
	fn blocks_val(&self) -> Option<u64> {
		self.meta_ok()
			.filter(|_| self.typ != Typ::Dir)
			.map(|meta| meta.blocks())
	}

	/// Get the value of the system time field specified by `field`.
	fn time_val(&self, field: DetailField) -> Option<SystemTime> {
		self.meta_ok().and_then(|meta| {
			match field {
				DetailField::Atime => meta.accessed(),
				DetailField::Btime => meta.created(),
				DetailField::Ctime => meta.c_time(),
				DetailField::Mtime => meta.modified(),
				_ => unreachable!("src/traits/det.rs / impl Detail for Node / time_val"),
			}
			.ok()
		})
	}

	/// Get the name of the user that owns this node, if known.
	fn user_val(&self, owner_man: &mut OwnerMan) -> Option<String> {
		self.meta_ok()
			.and_then(|meta| owner_man.user(meta.uid()).name)
	}

	/// Get the name of the group that owns this node, if known.
	fn group_val(&self, owner_man: &mut OwnerMan) -> Option<String> {
		self.meta_ok()
			.and_then(|meta| owner_man.group(meta.gid()).name)
	}

	// ===========
	// Renderables
	// ===========

	/// Get the device number, not the human-readable device name, of the node.
	///
	/// This function returns a marked-up string.
	fn dev(&self, entry_const: &EntryConst) -> Option<String> {
		self.meta_ok().map(|meta| {
			let dev = meta.dev().to_string();
			let directives = &entry_const.dev_style;
			format!("<{directives}>{dev}</>")
		})
	}

	/// Get the inode number of the node.
	///
	/// This function returns a marked-up string.
	fn ino(&self, entry_const: &EntryConst) -> Option<String> {
		self.meta_ok().map(|meta| {
			let ino = meta.ino().to_string();
			let directives = &entry_const.ino_style;
			format!("<{directives}>{ino}</>")
		})
	}

	/// Get the number of hard links pointing to the node.
	///
	/// Usually files have 1 hard link and directories have more than 1. So this
	/// function highlights files with more than 1 and directories with 1 hard
	/// link.
	///
	/// This function returns a marked-up string.
	fn nlink(&self, entry_const: &EntryConst) -> Option<String> {
		self.meta_ok()
			.map(|meta| entry_const.nlink_styles.format(meta.nlink(), &self.typ))
	}

	/// Get the symbolic representation of the permissions of the node.
	///
	/// This function returns a marked-up string.
	fn perm(&self, entry_const: &EntryConst) -> Option<String> {
		self.meta_ok()
			.map(|meta| Perm::from(meta.mode()).sym(entry_const))
	}

	/// Get the octal representation of the permissions of a node.
	///
	/// This function returns a marked-up string.
	fn oct(&self, entry_const: &EntryConst) -> Option<String> {
		self.meta_ok()
			.map(|meta| Perm::from(meta.mode()).oct(entry_const))
	}

	/// Get the name of the user that owns this node. The name is highlighted if
	/// the owner is the current user.
	///
	/// This function returns a marked-up string.
	fn user(&self, owner_man: &mut OwnerMan, entry_const: &EntryConst) -> Option<String> {
		self.meta_ok()
			.map(|meta| owner_man.user(meta.uid()).name(entry_const))
	}

	/// Get the UID of the user that owns this node. The UID is highlighted if
	/// the owner is the current user.
	///
	/// This function returns a marked-up string.
	fn uid(&self, owner_man: &mut OwnerMan, entry_const: &EntryConst) -> Option<String> {
		self.meta_ok()
			.map(|meta| owner_man.user(meta.uid()).id(entry_const))
	}

	/// Get the name of the group that owns this node. The name is highlighted
	/// if the current user belongs to this group.
	///
	/// This function returns a marked-up string.
	fn group(&self, owner_man: &mut OwnerMan, entry_const: &EntryConst) -> Option<String> {
		self.meta_ok()
			.map(|meta| owner_man.group(meta.gid()).name(entry_const))
	}

	/// Get the GID of the group that owns this node. The GID is highlighted
	/// if the current user belongs to this group.
	///
	/// This function returns a marked-up string.
	fn gid(&self, owner_man: &mut OwnerMan, entry_const: &EntryConst) -> Option<String> {
		self.meta_ok()
			.map(|meta| owner_man.group(meta.gid()).id(entry_const))
	}

	/// Get the size of the file in bytes, optionally with higher units in
	/// powers of 2^10 or 10^3.
	///
	/// This function returns a marked-up string.
	fn size(&self, entry_const: &EntryConst) -> Option<String> {
		self.size_val()
			.map(|size| PLS.args.unit.size(size, entry_const))
	}

	/// Get the number of blocks occupied by the file.
	///
	/// This function returns a marked-up string.
	fn blocks(&self, entry_const: &EntryConst) -> Option<String> {
		self.blocks_val().map(|blocks| {
			let directives = &entry_const.blocks_style;
			format!("<{directives}>{blocks}</>")
		})
	}

	/// Get the chosen timestamp field.
	///
	/// This function returns a marked-up string.
	fn time(&self, field: DetailField, entry_const: &EntryConst) -> Option<String> {
		self.time_val(field).map(|time| {
			let mut dt: OffsetDateTime = time.into();
			match UtcOffset::current_local_offset() {
				Ok(offset) => dt = dt.to_offset(offset),
				Err(_) => {
					warn!("Could not determine UTC offset")
				}
			}
			let format = format_description::parse_borrowed::<2>(
				entry_const.timestamp_formats.get(&field).unwrap(),
			)
			.unwrap();
			dt.format(&format).unwrap()
		})
	}

	/// Get the git status of the file or directory.
	/// This function returns a marked-up string.
	fn git(&self, _entry_const: &EntryConst) -> Option<String> {
		// Convert to absolute path first
		let absolute_path = match self.path.canonicalize() {
			Ok(path) => path,
			Err(_) => return Some("  ".to_string()),
		};

		// Try to find the git repository from the node's path
		let repo = match Repository::discover(&absolute_path) {
			Ok(repo) => repo,
			Err(_) => return Some("  ".to_string()), // Not in a git repo
		};

		// Get the relative path from the repository root
		let repo_path = repo.workdir()?;
		let relative_path = match absolute_path.strip_prefix(repo_path) {
			Ok(path) => path,
			Err(_) => return Some("  ".to_string()),
		};

		let relative_path_str = relative_path.to_string_lossy();

		// Quick check if path is ignored using git's ignore rules
		if repo.is_path_ignored(&absolute_path).unwrap_or(false) {
			return Some(format!("<red>!!</>"));
		}

		// For files, get status efficiently using path-specific query
		if self.typ != crate::enums::Typ::Dir {
			// Use git2's status_file method for single file - much faster than repo.statuses()
			match repo.status_file(relative_path) {
				Ok(status) => {
					if status.is_empty() {
						Some("  ".to_string())
					} else {
						Some(self.format_git_status(status))
					}
				}
				Err(_) => Some("  ".to_string()),
			}
		} else {
			// For directories, use optimized directory status check
			// First check if the directory itself has a status
			match repo.status_file(relative_path) {
				Ok(status) if !status.is_empty() => {
					return Some(self.format_git_status(status));
				}
				_ => {}
			}

			// If directory itself has no status, check if it contains any modified files
			// Use a more targeted approach with pathspec to limit scope
			let mut status_opts = git2::StatusOptions::new();
			status_opts.include_untracked(true);
			status_opts.include_ignored(false); // Skip ignored files for performance
			status_opts.recurse_untracked_dirs(false); // Don't recurse deeply

			// Limit the status check to just this directory and immediate children
			let pathspec = if relative_path_str.is_empty() {
				"*".to_string()
			} else {
				format!("{}/*", relative_path_str)
			};
			status_opts.pathspec(pathspec);

			match repo.statuses(Some(&mut status_opts)) {
				Ok(statuses) => {
					// Check if any files in this directory have changes
					let has_changes = statuses.iter().any(|entry| {
						if let Some(path) = entry.path() {
							// Check if this file is directly in our directory (not subdirectories)
							if relative_path_str.is_empty() {
								// Root directory - check if file is in root
								!path.contains('/')
							} else {
								// Check if file is directly in this directory
								path.starts_with(&format!("{}/", relative_path_str)) &&
								!path[relative_path_str.len() + 1..].contains('/')
							}
						} else {
							false
						}
					});

					if has_changes {
						Some(format!("<red> *</>"))
					} else {
						Some("  ".to_string())
					}
				}
				Err(_) => Some("  ".to_string()),
			}
		}
	}
}

impl<'pls> Node<'pls> {
	/// Format git status with proper color coding
	fn format_git_status(&self, status: Status) -> String {
		// Determine the staged (index) character
		let staged_char = if status.contains(Status::INDEX_NEW) {
			'A'
		} else if status.contains(Status::INDEX_MODIFIED) {
			'M'
		} else if status.contains(Status::INDEX_DELETED) {
			'D'
		} else if status.contains(Status::INDEX_RENAMED) {
			'R'
		} else if status.contains(Status::INDEX_TYPECHANGE) {
			'T'
		} else {
			' '
		};

		// Determine the unstaged (worktree) character
		let unstaged_char = if status.contains(Status::WT_NEW) {
			'?'
		} else if status.contains(Status::WT_MODIFIED) {
			'M'
		} else if status.contains(Status::WT_DELETED) {
			'D'
		} else if status.contains(Status::WT_RENAMED) {
			'R'
		} else if status.contains(Status::WT_TYPECHANGE) {
			'T'
		} else {
			' '
		};

		// Handle special cases
		if status.contains(Status::IGNORED) {
			return format!("<red>!!</>");
		}
		if status.contains(Status::CONFLICTED) {
			return format!("<red>UU</>");
		}
		if status.contains(Status::WT_NEW) && unstaged_char == '?' {
			return format!("<red>??</>");
		}

		// Format with colors: green for staged, red for unstaged
		let staged_formatted = if staged_char == ' ' {
			" ".to_string()
		} else {
			format!("<green>{}</>", staged_char)
		};

		let unstaged_formatted = if unstaged_char == ' ' {
			" ".to_string()
		} else {
			format!("<red>{}</>", unstaged_char)
		};

		format!("{}{}", staged_formatted, unstaged_formatted)
	}
}
