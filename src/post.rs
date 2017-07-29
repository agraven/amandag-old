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
	pub comment_count: i64,
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
		format!(r##"		<article>
			<h1><a href="/article/{id}">{title}</a></h1>
			<header>{time}</header>
			<header class="right">Filed under {category}</header>
			<p>{content}

			<footer>{count} comments</footer>
		</article>"##,
			id = self.id,
			title = self.title,
			time = time_string,
			category = self.category,
			content = self.content,
			count = self.comment_count)
	}
}
