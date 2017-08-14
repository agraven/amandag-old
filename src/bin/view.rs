extern crate amandag;
extern crate mysql;
extern crate time;

use amandag::cgi;
use amandag::Comment;
use amandag::CommentList;
use amandag::Post;
use amandag::strings;

fn format_document_header(title: &str, post_id: u64) -> String {
	format!(r##"Content-type: text/html; charset=utf-8
X-Powered-By: Rust/1.19.0
Content-Language: en

<!DOCTYPE html>
<html>
<head>
	<title>{title}</title>
	<meta name="author" content="Amanda Graven">
	<meta name="description" content="Personal homepage of Amanda Graven">

	<meta charset="UTF-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<link rel="stylesheet" type="text/css" href="/style.css">
	<script>
		function show(element) {lbrace}
			element.style.display = "block";
		{rbrace}
		function hide(element) {lbrace}
			element.style.display = "none";
		{rbrace}
		function send(form) {lbrace}
			function urlencodeFormData(fd) {lbrace}
				var s = '';
				function encode(s){lbrace}
					return encodeURIComponent(s).replace(/%20/g,'+');
				{rbrace}
				for(var pair of fd.entries()) {lbrace}
					if(typeof pair[1]=='string') {lbrace}
						s += (s?'&':'') + encode(pair[0])+'='+encode(pair[1]);
					{rbrace}
				{rbrace}
				return s;
			{rbrace}
			var data = new FormData(form);
			data.set("post", {post});
			for (var key of data.keys()) {lbrace}
				console.log('Key: ' + key);
			{rbrace}

			var request = new XMLHttpRequest();
			request.open("POST", "/comment.cgi", true);
			request.setRequestHeader("Content-Type", "application/x-www-form-urlencoded");
			request.onreadystatechange = function() {lbrace}
				if (this.readyState == 4 && this.status == 200) {lbrace}
					form.parentElement.innerHTML += this.responseText;
				{rbrace}
			{rbrace}
			request.send(urlencodeFormData(data));
		{rbrace}
	</script>
</head>
<body>
	<div id="headbar">
		<div id="headbar-content">
			<div class="title" style="float: left;">Amanda's terrible homepage</div>
			<ul class="navbar">
				<li><a href="/">Home</a></li>
				<li><a href="/about">About</a></li>
			</ul>
		</div>
	</div>
	<div id="easter-egg"> </div>
	<main>
		<div id="body-title">
			<div class="title">Amanda's terrible homepage</div>
		</div>"##,
		title = title,
		post = post_id,
		lbrace = '{',
		rbrace = '}'
	)
}

fn main() {
	// Get map of GET request and get id
	let id: i64 = cgi::get_get_member(String::from("id"))
		.unwrap_or(String::new()).parse().unwrap_or(-1);

	// Establish connection to MySQL server
	let pool = mysql::Pool::new("mysql://readonly:1234@localhost:3306/amandag")
		.expect("Failed to connect to database");
	// Get first (and hopefully only) article result
	let post: Post = if let Some(row) =
		pool.first_exec(
			"SELECT id, title, content, post_time, edit_time, category \
				FROM posts WHERE id = ?",
			(id,)
		).expect("Failed to get data from database") {
			// Bind values from row
			let (id, title, content, post_time, edit_time, category) =
				mysql::from_row_opt(row).unwrap_or( (
					0,
					String::from("Error!"),
					String::from("Failed to display article: \
						Error while fetching from database"),
					time::get_time(), time::get_time(),
					String::from("Error")
				));
			// Get amount of comments
			let comment_count = if let Some(row) =
				pool.first_exec(
					"SELECT COUNT(*) AS comment_count \
						FROM comments WHERE post_id = ?",
					(id,)
				).unwrap() {
					mysql::from_row_opt(row).unwrap_or(0)
				} else { 0 };
			Post { id, title, content, post_time, edit_time, category, comment_count }
		} else {
			Post {
				id: 0,
				title: String::from("Invalid id"),
				content:
					String::from("Failed to display article: \
					An article was requested with an id that doesn't exist."),
				post_time: time::get_time(),
				edit_time: time::get_time(),
				category: String::from("Error message"),
				comment_count: 0,
			}
		};

    let comments: Vec<Comment> =
		pool.prep_exec(
			"SELECT id, author, content, post_time, parent_id \
				FROM comments WHERE post_id = ?",
			(id,)
		).map(|result| { result.map(|x| x.unwrap()).map(|row| {
			let (id, author, content, post_time, parent_id) = mysql::from_row(row);
			Comment {id, author, content, post_time, parent_id}
        }).collect()
    }).unwrap();

	// print document
	println!("{}", format_document_header(&post.title, post.id));
	println!("{}", post.display());
	println!(r#"<article><form action="/comment.cgi" id="reply-root" method="post" onsubmit="send(this); return false;">
		<input name="parent" value="-1" style="display: none;">
		Name: <input type="text" name="name" required><br>
		<textarea name="content" required></textarea><br>
		<input type="button" value="Cancel" onclick="hide(this.parentElement)">
		<button type="submit">Submit</button>
	</form></article>"#);
	println!("{}", comments.display());
	println!("{}", strings::DOCUMENT_FOOTER);
}
