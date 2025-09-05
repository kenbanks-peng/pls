use crate::args::{Group, Input};
use crate::config::{Args, ConfMan};
use crate::fmt::render;
use crate::models::{GitMan, OwnerMan, Window};

/// Represents the entire application state.
///
/// This struct also holds various globals that are used across the
/// application.
#[derive(Default)]
pub struct Pls {
	/// configuration manager for `.pls.yml` files
	pub conf_man: ConfMan,
	/// command-line arguments
	pub args: Args,
	/// whether the terminal supports Kitty's terminal graphics protocol
	pub supports_gfx: bool,
	/// the width and height of a terminal cell in pixels
	pub window: Option<Window>,
}

impl Pls {
	/// Handle the `pls` command and its subcommands.
	///
	/// This is the entrypoint of the application that takes over the
	/// control from `main`.
	pub fn cmd(&self) {
		// TODO: Handle subcommands.
		self.run();
	}

	/// Run `pls`.
	///
	/// This is the entrypoint of the `Pls` class, and once control is passed
	/// to it from `main`, it handles everything.
	///
	/// The primary function of this method is to organise the input list of
	/// paths into groups and then delegate to each group the job of listing
	/// their entries and rendering the layout.
	fn run(&self) {
		let inputs: Vec<_> = self
			.args
			.paths
			.iter()
			.filter_map(|path| {
				let input = Input::new(path, &self.conf_man);
				match input {
					Ok(input) => Some(input),
					Err(exc) => {
						let loc = render(format!("<bold>{}</>", path.display()));
						println!("{loc}:");
						println!("\t{exc}");
						None
					}
				}
			})
			.collect();

		let show_title = self.args.paths.len() > 1;
		let groups = Group::partition(inputs, &self.conf_man);

		groups
			.iter()
			.map(|group| group.render(show_title, &mut OwnerMan::default(), &mut GitMan::default()))
			.filter_map(|res| res.err())
			.for_each(|res| println!("{res}"));
	}
}
