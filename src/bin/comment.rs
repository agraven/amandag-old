extern crate amandag;
extern crate mysql;
extern crate time;

use amandag::cgi;
use amandag::Comment;

use std::fs::File;
use std::io::Read;
use std::io::Write;

fn main() {
	// TODO: end-user friendly error messages
	if !cgi::request_method_is("POST") {
		panic!("Wrong request method");
	}
	let post_map = cgi::get_post().unwrap();
	let mut file = File::create("debug.log").unwrap();
	write!(&mut file, "{:?}", post_map).unwrap();

	// Get values
	let author = post_map.get("name").expect("Missing name").to_string();
	let content = post_map.get("content").expect("Missing content").to_string();
	let post_id = post_map.get("post").expect("Missing post id").parse::<u64>().unwrap();
	let parent_id = post_map.get("parent").expect("Missing parent id").parse::<i64>().unwrap();

	let mut pw_buf = Vec::new();
	File::open("secret/password").unwrap().read_to_end(&mut pw_buf).unwrap();
	let password = String::from_utf8(pw_buf).unwrap();

	let options = format!("mysql://root:{}@localhost:3306/amandag", password);
	let pool = mysql::Pool::new(options).unwrap();
	let id: u64 = mysql::from_row(pool.first_exec(r#"SELECT min(unused) AS unused
		FROM (
			SELECT MIN(t1.id)+1 as unused
			FROM comments AS t1
			WHERE NOT EXISTS (SELECT * FROM comments AS t2 WHERE t2.id = t1.id+1)
			UNION
			SELECT 1
			FROM DUAL
			WHERE NOT EXISTS (SELECT * FROM comments WHERE id = 1)
		) AS subquery"#, ()).unwrap().unwrap());
	pool.prep_exec(
		"INSERT INTO comments (id, author, content, post_id, parent_id) \
		VALUES (?, ?, ?, ?, ?)",
		(id, &author, &content, post_id, parent_id)
	).unwrap();
	let post_time = time::get_time();
	
	println!(
		"Content-Type: text/html; charset=utf-8\n\n{}",
		Comment {id, author, content, post_time, parent_id}.display()
	);
}
