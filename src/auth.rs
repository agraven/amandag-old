use crypto::bcrypt::bcrypt;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use mysql;
use rand::{self, Rng};
use std::fs::File;
use std::io::Read;
use time;
use time::{Duration, Timespec};

use cgi;

const SALT_LENGTH: usize = 16;
const SESSID_LENGTH: usize = 64;

const ADDRESS: &'static str = "mysql://readonly:1234@localhost/amandag";
const INSERT_SESSION: &'static str = "INSERT INTO sessions (id, user, expiry) VALUES (?, ?, ?)";
const INSERT_USER: &'static str = "INSERT INTO users (id, pass, salt, name) VALUES (?, ?, ?, ?)";
const DELETE_SESSION: &'static str = "DELETE FROM sessions WHERE id = ?";
const SELECT_SESSION: &'static str = "SELECT id, user, expiry FROM sessions WHERE id = ?";
const SELECT_USER: &'static str = "SELECT pass, salt FROM users WHERE id = ?";

error_chain! {
	foreign_links {
		Sql(mysql::Error);
		Utf8(::std::string::FromUtf8Error);
        Io(::std::io::Error);
	}
	links {
		Cgi(cgi::Error, cgi::ErrorKind);
	}
	errors {
		ExpiredToken {
			description("token has expired"),
			display("Token has expired"),
		}
		InvalidToken {
			description("token is invalid"),
			display("Invalid login token"),
		}
		NoSuchToken {
			description("no such token"),
			display("Attemped to login with nonexistant token"),
		}
		NoSuchUser(s: String) {
			description("no such user"),
			display("The user {} doesn't exist", s),
		}
		WrongPassword {
			description("wrong password"),
			display("Wrong password"),
		}
	}
}

pub struct User {
	pub id: String,
	pub pass: String,
}

pub struct Session {
	pub id: String,
	pub user: String,
	pub expiry: Timespec,
}

fn rand_str(len: usize) -> String {
	rand::thread_rng().gen_ascii_chars().take(len).collect()
}

fn hash(key: &str, salt: &str) -> String {
	// Hash password to mitigate DoS
	let hex = {
		let mut digest = Sha256::new();
		digest.input_str(&key);
		digest.result_str()
	};
	// Salt
	let mut output = [0u8; 24];
	bcrypt(12, salt.as_bytes(), hex.as_bytes(), &mut output);
	// TODO: encrypt with pepper
	String::from_utf8_lossy(&output).to_string()
}

/// Verify a login session
pub fn auth(sessid: &str) -> Result<Session> {
	let pool = mysql::Pool::new(ADDRESS)?;
	if let Some(row) = pool.first_exec(SELECT_SESSION, (sessid,))? {
		let (id, user, expiry): (String, String, Timespec) =
			mysql::from_row(row);
		if expiry < time::get_time() {
			pool.first_exec(DELETE_SESSION, (sessid,))?;
			Err(ErrorKind::ExpiredToken.into())
		} else {
			return Ok(
				Session { id: id.to_owned(), user: user.to_owned(), expiry },
			);
		}
	} else {
		Err(ErrorKind::NoSuchToken.into())
	}
}

/// Sign in user with password, creating a login token
pub fn login(user: &str, password: &str) -> Result<Session> {
	let dbpassword = String::from_utf8(
		File::open("secret/db-submit")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;
	let pool = mysql::Pool::new(
		format!("mysql://submit:{}@localhost/amandag", dbpassword),
	)?;
	if let Some(row) = pool.first_exec(SELECT_USER, (user,))? {
		let (pass, salt): (String, String) = mysql::from_row(row);
		if hash(&password, &salt) == pass {
			// Correct password, create token
			let id = rand_str(SESSID_LENGTH);
			let expiry = time::get_time() + Duration::days(30);
			pool.prep_exec(INSERT_SESSION, (&id, &user, expiry))?;
			Ok(Session { id, user: user.to_owned(), expiry })
		} else {
			Err(ErrorKind::WrongPassword.into())
		}
	} else {
		Err(ErrorKind::NoSuchUser(user.to_owned()).into())
	}
}

pub fn logout(session: &str) -> Result<()> {
	let dbpassword = String::from_utf8(
		File::open("secret/db-submit")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;
	let pool = mysql::Pool::new(
		format!("mysql://submit:{}@localhost/amandag", dbpassword),
	)?;
	pool.prep_exec(DELETE_SESSION, (session,))?;
	Ok(())
}

pub fn create(user: &str, password: &str, name: &str) -> Result<()> {
	let dbpassword = String::from_utf8(
		File::open("secret/db-submit")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;
	let pool = mysql::Pool::new(
		format!("mysql://submit:{}@localhost/amandag", dbpassword),
	)?;
	let salt = rand_str(SALT_LENGTH);
	pool.prep_exec(
		INSERT_USER,
		(user, hash(&password, &salt), salt, name),
	)?;
	Ok(())
}

pub fn get_session() -> Result<Session> {
	let cookies = cgi::get_cookies()?;
	if let Some(id) = cookies.get("session") {
		auth(&id)
	} else {
		Ok(Session {
			id: String::new(),
			user: String::from("guest"),
			expiry: time::get_time(),
		})
	}
}
