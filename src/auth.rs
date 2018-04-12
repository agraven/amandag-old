use crypto::bcrypt::bcrypt;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use gotham::state::FromState;
use gotham::state::State;
use hyper::header::{Cookie, Headers};
use mysql;
use rand::{self, Rng};
use std::fs::File;
use std::io::Read;
use time;
use time::{Duration, Timespec};

use error::{Error, Result};

const SALT_LENGTH: usize = 16;
const SESSID_LENGTH: usize = 64;

const ADDRESS: &'static str = "mysql://readonly:1234@localhost/amandag";
const INSERT_SESSION: &'static str =
	"INSERT INTO sessions (id, user, expiry) VALUES (?, ?, ?)";
const INSERT_USER: &'static str =
	"INSERT INTO users (id, pass, salt, name) VALUES (?, ?, ?, ?)";
const DELETE_SESSION: &'static str = "DELETE FROM sessions WHERE id = ?";
const SELECT_SESSION: &'static str =
	"SELECT id, user, expiry FROM sessions WHERE id = ?";
const SELECT_USER: &'static str = "SELECT pass, salt FROM users WHERE id = ?";

#[derive(Clone)]
pub struct Session {
	pub id: String,
	pub user: String,
	pub expiry: Timespec,
}

fn rand_str(len: usize) -> String {
	rand::thread_rng()
		.gen_ascii_chars()
		.take(len)
		.collect()
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
			Err(Error::ExpiredToken)
		} else {
			return Ok(Session {
				id: id.to_owned(),
				user: user.to_owned(),
				expiry,
			});
		}
	} else {
		Err(Error::NoSuchToken)
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
	let pool = mysql::Pool::new(format!(
		"mysql://submit:{}@localhost/amandag",
		dbpassword
	))?;
	if let Some(row) = pool.first_exec(SELECT_USER, (user,))? {
		let (pass, salt): (String, String) = mysql::from_row(row);
		if hash(&password, &salt) == pass {
			// Correct password, create token
			let id = rand_str(SESSID_LENGTH);
			let expiry = time::get_time() + Duration::days(30);
			pool.prep_exec(INSERT_SESSION, (&id, &user, expiry))?;
			Ok(Session {
				id,
				user: user.to_owned(),
				expiry,
			})
		} else {
			Err(Error::WrongPassword)
		}
	} else {
		Err(Error::NoSuchUser)
	}
}

pub fn logout(session: &str) -> Result<()> {
	let dbpassword = String::from_utf8(
		File::open("secret/db-submit")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;
	let pool = mysql::Pool::new(format!(
		"mysql://submit:{}@localhost/amandag",
		dbpassword
	))?;
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
	let pool = mysql::Pool::new(format!(
		"mysql://submit:{}@localhost/amandag",
		dbpassword
	))?;
	let salt = rand_str(SALT_LENGTH);
	pool.prep_exec(
		INSERT_USER,
		(user, hash(&password, &salt), salt, name),
	)?;
	Ok(())
}

pub fn get_session(state: &State) -> Result<Session> {
	if let Some(cookies) = Headers::borrow_from(state).get::<Cookie>() {
		if let Some(id) = cookies.get("token") {
			return auth(&id);
		}
	}
	Ok(Session {
		id: String::new(),
		user: String::from("guest"),
		expiry: time::get_time(),
	})
}
