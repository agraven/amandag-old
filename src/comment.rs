extern crate time;

use cgi::Encode;

use self::time::Timespec;

#[derive(Clone)]
pub struct Comment {
	pub id: u64,
	pub author: String,
	pub user: String,
	pub content: String,
	pub post_time: Timespec,
	pub parent_id: i64,
}

pub trait CommentList {
	fn display(&self) -> String;
	fn display_from_root(&self, root: i64) -> String;
	fn with_parent_id(&self, parent_id: i64) -> Vec<Comment>;
}
fn color_name(id: u64) -> String {
	String::from(match id % 7 {
		0 => "red",
		1 => "green",
		2 => "blue",
		3 => "ice",
		4 => "yellow",
		5 => "brown",
		6 => "purple",
		_ => unreachable!(),
	})
}

impl Comment {
	pub fn display(&self) -> String {
		// format
		format!(
			include_str!("web/comment-list.html"),
			author = self.author,
			user = self.user,
			color = color_name(self.id),
			content = self.content.render_html(),
			form = "this.parentElement.parentElement.nextElementSibling",
			id = self.id,
			time = time::at(self.post_time).ctime(),
			children = "",
		)
	}
}

impl CommentList for Vec<Comment> {
	fn display(&self) -> String { self.display_from_root(-1) }
	fn display_from_root(&self, root: i64) -> String {
		let mut string = String::new();
		// Return a new string to stop recursion if no children found
		if self.len() == 0 {
			return string;
		}
		// Get comments with desired parent comment
		for comment in self.with_parent_id(root) {
			string.push_str(&format!(
				include_str!("web/comment-list.html"),
				author = comment.author,
				user = comment.user,
				color = color_name(comment.id),
				content = comment.content.render_html(),
				form = "this.parentElement.parentElement.nextElementSibling",
				id = comment.id,
				time = time::at(comment.post_time).ctime(),
				children = self.display_from_root(comment.id as i64),
			));
		}
		string
	}
	fn with_parent_id(&self, parent_id: i64) -> Vec<Comment> {
		self.iter()
			.cloned()
			.filter(|c| c.parent_id == parent_id)
			.collect()
	}
}
