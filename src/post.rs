extern crate time;

use self::time::Timespec;

// TODO: rename to Article?
pub struct Post {
	pub id: u64,
	pub title: String,
	pub content: String,
	pub post_time: Timespec,
	pub edit_time: Timespec,
	pub category: String,
}

impl Post {
	pub fn display(&self) -> String {
		// Build submission string so edit time only shows when greater than post time
		let time_string = if self.post_time < self.edit_time {
			format!("Submitted {}, Last edited {}", time::at(self.post_time).ctime(), time::at(self.edit_time).ctime())
		} else {
			format!("Submitted {}", time::at(self.post_time).ctime())
		};
		// Print article
		format!("\t\t<article>
			<h1>{}</h1>
			<header>{}</header>
			<header class=\"right\">Filed under {}</header>
			<p>{}\n\t\t</article>",
		self.title, time_string, self.category, self.content)
	}
}
