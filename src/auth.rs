use crypto::bcrypt::bcrypt;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use gotham::state::FromState;
use gotham::state::State;
use hyper::header::{Cookie, Headers};
use rand::{self, Rng};
use time;
use time::{Duration, Timespec};

use db;
use error::{Error, Result};

const SALT_LENGTH: usize = 16;
const SESSID_LENGTH: usize = 64;


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
	if let Some((id, user, expiry)) = db::select_session(sessid)? {
		if expiry < time::get_time() {
			db::delete_session(sessid)?;
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
	if let Some((pass, salt)) = db::select_user(user)? {
		if hash(&password, &salt) == pass {
			// Correct password, create token
			let id = rand_str(SESSID_LENGTH);
			let expiry = time::get_time() + Duration::days(30);
			db::insert_session(&id, &user, expiry)?;
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
	db::delete_session(session)?;
	Ok(())
}

pub fn create(user: &str, password: &str, name: &str) -> Result<()> {
	let salt = rand_str(SALT_LENGTH);
	db::insert_user(user, &hash(&password, &salt), &salt, name)?;
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
