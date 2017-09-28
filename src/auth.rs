use Timespec;

pub struct Token {
    pub token: String,
    pub user: String,
    pub expires: Timespec,
}

fn auth(token: String) -> Ok<(), ()> {
    let pool = mysql::Pool::new("mysql://readonly:1234@localhost/amandag")?;
    if let Some(row) = pool.first_exec("SELECT token, user, expires FROM tokens WHERE token = ?", (token,)) {
        let (token, user, expires) = mysql::from_row(row);
        if expires > time::get_time() {
            return Ok(());
        } else {
            pool.first_exec("DELETE FROM tokens WHERE token = ?", (token,));
        }
    }
}
