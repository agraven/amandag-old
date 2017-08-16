extern crate amandag;
extern crate mysql;
extern crate time;

use amandag::cgi;
use amandag::Comment;

use std::fs::File;
use std::io::Read;

macro_rules! impl_error {
	[ $( ($l:ident, $f:ty) ),* ] => {
		$(
			impl From<$f> for Error {
				fn from(err: $f) -> Error {
					Error::$l(err)
				}
			}
		)*
	}
}

enum Error {
	IoError(std::io::Error),
	MethodError,
	MissingError(&'static str),
	ParseIntError(std::num::ParseIntError),
	SqlError(mysql::Error),
	Utf8Error(std::string::FromUtf8Error),
}
use Error::*;

impl_error![
	(IoError, std::io::Error),
	(ParseIntError, std::num::ParseIntError),
	(SqlError, mysql::Error),
	(Utf8Error, std::string::FromUtf8Error)
];

fn main() {
	match run() {
		Ok(()) => (),
		Err(e) => match e {
			IoError(_) => panic!("I/O Error"),
			MethodError => panic!("Wrong request method"),
			MissingError(p) => panic!("Missing parameter {}", p),
			ParseIntError(_) => panic!("Int parsing error"),
			SqlError(_) => panic!("Database error"),
			Utf8Error(_) => panic!("UTF-8 parsing error"),
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
	let post_id = get("post")?.parse::<i64>()?;
	let parent_id = get("parent")?.parse::<i64>()?;

	let mut pw_buf = Vec::new();
	File::open("secret/password")?.read_to_end(&mut pw_buf)?;
	let password = String::from_utf8(pw_buf)?;

	let options = format!("mysql://root:{}@localhost:3306/amandag", password);
	let pool = mysql::Pool::new(options)?;
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
