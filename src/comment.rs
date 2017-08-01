extern crate time;

use self::time::Timespec;

pub struct Comment<'a> {
	pub id: u64,
	pub author: &'a str,
	pub content: &'a str,
	pub post_time: Timespec,
	pub parent_id: u64,
}

pub fn display_comments(comments: Vec<Comment>) {
	for comment in comments.iter().map(|c| if c.parent_id == 0 {c}).collect() {
		println!(r#"<div class="comment"> "#);
	}
}
