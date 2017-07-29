extern crate mysql;
extern crate amandag;
extern crate time;

use amandag::Post;
use amandag::strings;
use amandag::cgi;

fn main() {
	// Get map of GET request and get id
	let mut id: i64 = -1;
	if let Some(get_map) = cgi::get_get() {
		if let Some(id_string) = get_map.get("id") {
			id = id_string.parse().unwrap_or(-1);
		} 
	}

	// Establish connection to MySQL server
	let pool = mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag").expect("Failed to connect to database");
	// Get first (and hopefully only) result
	let post: Post = if let Some(row) =
		pool.first_exec(format!("SELECT id, title, content, post_time, edit_time, category FROM posts WHERE id = {}", id), ())
			.expect("Failed to get data from database")
		{
			let (id, title, content, post_time, edit_time, category) = mysql::from_row_opt(row)
				.unwrap_or( (
					0,
					String::from("Error!"),
					String::from("Failed to display article: Error while fetching from database"),
					time::get_time(), time::get_time(),
					String::from("Error"))
				);
			Post {
				id: id,
				title: title,
				content: content,
				post_time: post_time,
				edit_time: edit_time,
				category: category,
				comment_count: 0,
			}
		} else {
			Post {
				id: 0,
				title: "Invalid id".to_string(),
				content: "Failed to display article: An article was requested with an id that doesn't exist.".to_string(),
				post_time: time::get_time(),
				edit_time: time::get_time(),
				category: "Error message".to_string(),
				comment_count: 0,
			}
		};
	// print document
	println!("{}", strings::format_document_header(post.title.as_str()));
	println!("{}", post.display());
	println!("{}", strings::DOCUMENT_FOOTER);
}
