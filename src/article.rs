extern crate time;

use cgi::Encode;

use self::time::Timespec;

pub struct Article {
	pub id: u64,
	pub title: String,
	pub content: String,
	pub post_time: Timespec,
	pub edit_time: Timespec,
	pub category: String,
	pub comment_count: i64,
}

impl Article {
	pub fn display(&self) -> String {
		// Only show edit time when it's greater than post time
		let time_string = if self.post_time < self.edit_time {
			format!(
				"Submitted {}, Last edited {}",
				time::at(self.post_time).ctime(),
				time::at(self.edit_time).ctime()
			)
		} else {
			format!("Submitted {}", time::at(self.post_time).ctime())
		};
		// Print article
		format!(
			include_str!("web/article.html"),
			id = self.id,
			title = self.title,
			time = time_string,
			category = self.category,
			content = self.content.render_html(),
			count = self.comment_count
		)
	}
}
