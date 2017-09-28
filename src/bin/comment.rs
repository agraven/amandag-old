#[macro_use]
extern crate amandag;

use amandag::captcha;
use amandag::cgi;
use amandag::Comment;
use amandag::mysql;
use amandag::time;

use std::fs::File;
use std::io::Read;

enum Error {
	CaptchaError(captcha::Error),
	IoError(std::io::Error),
	MethodError,
	MissingError(&'static str),
	ParseIntError(std::num::ParseIntError),
	SqlError(mysql::Error),
	Utf8Error(std::string::FromUtf8Error),
}
use Error::*;

impl_error![
	(CaptchaError, captcha::Error),
	(IoError, std::io::Error),
	(ParseIntError, std::num::ParseIntError),
	(SqlError, mysql::Error),
	(Utf8Error, std::string::FromUtf8Error)
];

fn main() {
	match run() {
		Ok(()) => (),
		Err(e) => match e {
			CaptchaError(e) => panic!("reCAPTCHA failed: {}", e),
			IoError(e) => panic!("I/O Error: {}", e),
			MethodError => panic!("Wrong request method"),
			MissingError(p) => panic!("Missing parameter {}", p),
			ParseIntError(e) => panic!("Int parsing error: {}", e),
			SqlError(e) => panic!("Database error: {}", e),
			Utf8Error(e) => panic!("UTF-8 parsing error: {}", e),
		}
	}
}

fn run() -> Result<(), Error> {
	// TODO: end-user friendly error messages
	if !cgi::request_method_is("POST") {
		return Err(MethodError);
	}
	let post_map = cgi::get_post().unwrap();
	// Get values
	let get = |key: &'static str| -> Result<&String, Error> {
		post_map.get(key).ok_or(MissingError(key))
	};
	let author = get("name")?.clone();
	let content = get("content")?.clone();
	let post_id = get("id")?.parse::<i64>()?;
	let parent_id = get("parent")?.parse::<i64>()?;
	let response = get("g-recaptcha-response")?;
	let secret = String::from_utf8(
		File::open("secret/comment-captcha")?
			.bytes()
			.map(|b| b.unwrap())
			.collect()
	)?;

	captcha::verify(&secret, &response)?;

	let password = String::from_utf8(
		File::open("secret/password")?
			.bytes()
			.map(|b| b.unwrap())
			.collect()
	)?;

	let options = format!("mysql://root:{}@localhost:3306/amandag", password);
	let pool = mysql::Pool::new(options)?;
    // Get a unique id
	let id: u64 = mysql::from_row(pool.first_exec(r#"SELECT min(unused) AS unused
		FROM (
			SELECT MIN(t1.id)+1 as unused
			FROM comments AS t1
			WHERE NOT EXISTS (SELECT * FROM comments AS t2 WHERE t2.id = t1.id+1)
			UNION
			SELECT 1
			FROM DUAL
			WHERE NOT EXISTS (SELECT * FROM comments WHERE id = 1)
		) AS subquery"#, ())?.unwrap());
	pool.prep_exec(
		"INSERT INTO comments (id, author, content, post_id, parent_id) \
		VALUES (?, ?, ?, ?, ?)",
		(id, &author, &content, post_id, parent_id)
	)?;
	let post_time = time::get_time();

	println!(
		"Content-Type: text/html; charset=utf-8\n\n{}",
		Comment {id, author, content, post_time, parent_id}.display()
	);
	Ok(())
}
