extern crate time;

use self::time::Timespec;

#[derive(Clone)]
pub struct Comment {
	pub id: u64,
	pub author: String,
	pub content: String,
	pub post_time: Timespec,
	pub parent_id: u64,
}

pub trait CommentList {
	fn display(&self) -> String;
	fn display_from_root(&self, root: u64) -> String;
    fn with_parent_id(&self, parent_id: u64) -> Vec<Comment>;
}

impl CommentList for Vec<Comment> {
	fn display(&self) -> String {
		self.display_from_root(0)
	}
	fn display_from_root(&self, root: u64) -> String {
		let mut string = String::new();
		// Return a new string to stop recursion if no children found
		if self.len() == 0 { return string }
        // Get comments with desired parent comment
		for comment in self.with_parent_id(root) {
			string.push_str(&format!("<div class=\"comment\">\
			<h3>{author}</h3><div class=\"time\">{time}\n\t<p>{content}{children}\n</div>",
                author = comment.author,
				time = time::at(comment.post_time).ctime(),
                content = comment.content,
                children = self.display_from_root(comment.id)
			));
		}
		string
	}
	fn with_parent_id(&self, parent_id: u64) -> Vec<Comment> {
		self.iter().cloned().filter(|c| c.parent_id == parent_id).collect::<Vec<Comment>>()
	}
}

/*impl<'l> CommentList for Vec<&Comment<'l>> {
	fn display(&self) -> String {
		self.display_from_root(0)
	}
	fn display_from_root(&self. root: u64) -> String {
		let mut string = String::new()
    }
}*/
