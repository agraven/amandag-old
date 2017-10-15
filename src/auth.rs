use crypto::bcrypt::bcrypt;
use crypto::hmac::Hmac;
use crypto::mac::{Mac, MacResult};
use crypto::sha2::Sha512;
use crypto::digest::Digest;
use rand::{self, Rng};
use time;
use time::{Duration, Timespec};
use mysql;

const SALT_LENGTH: usize = 16;
const TOKEN_LENGTH: usize = 64;

const ADDRESS: &'static str = "mysql://readonly:1234@localhost/amandag";
const INSERT_TOKEN: &'static str = "INSERT INTO tokens (token, hash, user, expires) VALUES (?, ?, ?)";
const INSERT_USER: &'static str = "INSERT INTO users (id, pass, salt, name) VALUES (?, ?, ?, ?)";
const DELETE_TOKEN: &'static str = "DELETE FROM tokens WHERE token = ?";
const SELECT_TOKEN: &'static str = "SELECT token, expiry FROM tokens WHERE hash = ?";
const SELECT_USER: &'static str = "SELECT pass, salt FROM users WHERE id = ?";

error_chain! {
    foreign_links {
        Sql(mysql::Error);
        Utf8(::std::string::FromUtf8Error);
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

pub struct Token {
    pub hash: String,
    pub user: String,
    pub expiry: Timespec,
}

fn rand_str_len(len: usize) -> String {
    rand::thread_rng().gen_ascii_chars().take(len).collect()
}

fn hash(key: &str, salt: &str) -> String {
    // Hash password to mitigate DoS
    let hex = {
        let mut digest = Sha512::new();
        digest.input_str(key);
        digest.result_str()
    };
    // Salt
    let mut output = [0u8; 24];
    bcrypt(12, salt.as_bytes(), hex.as_bytes(), &mut output);
    // TODO: encrypt with pepper
    String::from_utf8_lossy(&output).to_string()
}

fn hmac(key: &str, msg: &str) -> MacResult {
    let mut mac = Hmac::new(Sha512::new(), key.as_bytes());
    mac.input(msg.as_bytes());
    mac.result()
}

fn verify_hmac(result: MacResult, hash: &str) -> bool {
    result == MacResult::new_from_owned(hash.to_owned().into_bytes())
}

/// Verify a login token
pub fn auth(user: &str, hash: &str) -> Result<Token> {
    let pool = mysql::Pool::new(ADDRESS)?;
    if let Some(row) = pool.first_exec(SELECT_TOKEN, (hash,))? {
        let (token, expiry): (String, Timespec) = mysql::from_row(row);
        if !verify_hmac(hmac(&token, user), hash) {
            return Err(ErrorKind::InvalidToken.into());
        } else if expiry < time::get_time() {
            pool.first_exec(DELETE_TOKEN, (token,))?;
            Err(ErrorKind::ExpiredToken.into())
        } else {
            return Ok(Token {hash: hash.to_owned(), user: user.to_owned(), expiry});
        }
    } else {
        Err(ErrorKind::NoSuchToken.into())
    }
}

/// Sign in user with password, creating a login token
pub fn login(user: &str, password: &str) -> Result<Token> {
    let pool = mysql::Pool::new(ADDRESS)?;
    if let Some(row) = pool.first_exec(SELECT_USER, (user,))? {
        let (pass, salt): (String, String) = mysql::from_row(row);
        if hash(password, &salt) == pass {
            // Correct password, create token
            let token = rand_str_len(TOKEN_LENGTH);
            let hash = String::from_utf8(hmac(&token, user).code().to_owned())?;
            let expiry = time::get_time() + Duration::days(30);
            pool.prep_exec(INSERT_TOKEN, (&token, &user, expiry))?;
            Ok(Token {hash, user: user.to_owned(), expiry})
        } else {
            Err(ErrorKind::WrongPassword.into())
        }
    } else {
        Err(ErrorKind::NoSuchUser(user.to_owned()).into())
    }
}

pub fn create(user: &str, password: &str, name: &str) -> Result<()> {
    let pool = mysql::Pool::new(ADDRESS)?;
    let salt = rand_str_len(SALT_LENGTH);
    pool.prep_exec(INSERT_USER, (user, password, salt, name))?;
    Ok(())
}
