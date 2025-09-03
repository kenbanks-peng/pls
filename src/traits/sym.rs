use crate::enums::{SymTarget, Typ};
use crate::exc::Exc;
use crate::models::Node;
use std::fs;

pub trait Sym {
	fn target(&self) -> Option<SymTarget<'_>>;
}

impl Sym for Node<'_> {
	/// Get the target destination of the node.
	///
	/// If the node is not a symlink, the target is `None`. If the node is a
	/// symlink, the target is a variant of [`SymTarget`], wrapped in `Some`.
	fn target(&self) -> Option<SymTarget<'_>> {
		if self.typ != Typ::Symlink {
			return None;
		}

		let target_path = match fs::read_link(&self.path) {
			Ok(path) => path,
			Err(err) => return Some(SymTarget::Error(Exc::Io(err))),
		};

		// Normalise the symlink path. This process handles symlink that use a
		// relative path as target.
		let abs_target_path = if target_path.is_absolute() {
			target_path.clone()
		} else if let Some(parent) = self.path.parent() {
			parent.join(&target_path)
		} else {
			self.path.join(&target_path)
		};

		let target = match abs_target_path.try_exists() {
			Err(err) => match err.raw_os_error() {
				// 62: 'Too many levels of symbolic links'
				// 40: 'Symbolic link loop'
				Some(62) | Some(40) => SymTarget::Cyclic(target_path),
				_ => SymTarget::Error(Exc::Io(err)),
			},
			Ok(true) => SymTarget::Ok(Box::new(
				Node::new(&abs_target_path).symlink(target_path.to_string_lossy().to_string()),
			)),
			Ok(false) => SymTarget::Broken(target_path),
		};
		Some(target)
	}
}
