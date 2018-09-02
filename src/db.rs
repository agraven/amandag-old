use mysql;
use mysql::Pool;

use time;

use std::fs::File;
use std::io::Read;
use std::sync::Mutex;

use article::Article;
use comment::Comment;
use error::{Error, Result};

lazy_static! {
	static ref POOL: Mutex<mysql::Pool> =
		Mutex::new(mysql::Pool::new(opts().unwrap()).unwrap());
}

fn opts() -> Result<mysql::Opts> {
	let password = String::from_utf8(
		File::open("secret/db-submit")?
			.bytes()
			.map(|b| b.unwrap())
			.collect(),
	)?;
	let url = format!("mysql://submit:{}@localhost/amandag", password);
	Ok(mysql::Opts::from_url(&url).unwrap())
}

pub fn select_article(id: u64) -> Result<Article> {
	const SELECT_ARTICLE: &'static str =
		"SELECT id, title, content, post_time, edit_time, category \
		 FROM posts WHERE id = ?";
	let pool = POOL.lock().unwrap();
	let row = pool
		.first_exec(SELECT_ARTICLE, (id,))?
		.ok_or(Error::InvalidId(id as u64))?;
	let (id, title, content, post_time, edit_time, category) =
		mysql::from_row(row);
	Ok(Article {
		id,
		title,
		content,
		post_time,
		edit_time,
		category,
		comment_count: select_comment_count(&pool, id)? as i64,
	})
}

pub fn select_articles() -> Result<Vec<Article>> {
    const SELECT_ARTICLES: &str =
        "SELECT id, title, content, post_time, edit_time, category \
	 FROM posts \
	 ORDER BY post_time DESC";
    let pool = POOL.lock().unwrap();
    let articles = pool
        .prep_exec(SELECT_ARTICLES, ())
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(|row| {
                    let (id, title, content, post_time, edit_time, category) =
                        mysql::from_row(row);
                    let comment_count = select_comment_count(&pool, id).unwrap() as i64;
                    Article {
                        id,
                        title,
                        content,
                        post_time,
                        edit_time,
                        category,
                        comment_count,
                    }
                })
                .collect()
        })?;
    Ok(articles)
}

pub fn select_comment_count(pool: &Pool, id: u64) -> Result<u64> {
	const SELECT_COMMENT_COUNT: &str = "SELECT COUNT(*) AS comment_count \
	                                    FROM comments WHERE post_id = ?";
	let row = pool.first_exec(SELECT_COMMENT_COUNT, (id,))?.unwrap();
	Ok(mysql::from_row(row))
}

pub fn select_comments(article_id: u64) -> Result<Vec<Comment>> {
	const SELECT_COMMENTS: &str =
		"SELECT id, author, user, content, post_time, parent_id \
		 FROM comments WHERE post_id = ?";
	let pool = POOL.lock().unwrap();
	let comments = pool.prep_exec(SELECT_COMMENTS, (article_id,)).map(
		|result| {
			result
				.map(|x| x.unwrap())
				.map(|row| {
					let (id, author, user, content, post_time, parent_id) =
						mysql::from_row(row);
					Comment {
						id,
						author,
						user,
						content,
						post_time,
						parent_id,
					}
				})
				.collect()
		}
	)?;
	Ok(comments)
}

pub fn insert_article(title: &str, content: &str, category: &str) -> Result<()> {
    let pool = POOL.lock().unwrap();
    const INSERT_ARTICLE: &str =
        "INSERT INTO posts (title, content, category) VALUES (?, ?, ?)";
    pool.prep_exec(INSERT_ARTICLE, (title, content, category))?;
    Ok(())
}

pub fn insert_comment(
    user: &str,
    author: &str,
    content: &str,
    post_id: i64,
    parent_id: i64
) -> Result<u64> {
    const SELECT_UNUSED: &str = r#"SELECT min(unused) AS unused
        FROM (
            SELECT MIN(t1.id)+1 as unused
            FROM comments AS t1
            WHERE NOT EXISTS (SELECT * FROM comments AS t2 WHERE t2.id = t1.id+1)
            UNION
            SELECT 1
            FROM DUAL
            WHERE NOT EXISTS (SELECT * FROM comments WHERE id = 1)
        ) AS subquery"#;
    const INSERT_COMMENT: &str =
        "INSERT INTO comments (id, user, author, content, post_id, parent_id) \
		 VALUES (?, ?, ?, ?, ?, ?)";

    let pool = POOL.lock().unwrap();
	let id: u64 = mysql::from_row(pool.first_exec(SELECT_UNUSED, ())?.unwrap());

    pool.prep_exec(
        INSERT_COMMENT,
        (id, user, author, content, post_id, parent_id),
    )?;
    Ok(id)
}

pub fn update_article(
    title: &str,
    content: &str,
    category: &str,
    time: time::Timespec,
    id: u64
) -> Result<()> {
    const UPDATE_ARTICLE: &str =
        "UPDATE posts \
        SET title = ?, content = ?, category = ?, edit_time = ? \
        WHERE id = ?";
    let pool = POOL.lock().unwrap();
    pool.prep_exec(
        UPDATE_ARTICLE, (title, content, category, time, id)
    )?;
    Ok(())
}
