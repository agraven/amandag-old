pub fn format_document_header(title: &str) -> String {
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
	)
}
pub fn format_captcha_header(title: &str) -> String {
	format!(r##"Content-type: text/html; charset=utf-8
X-Powered-By: Rust/1.16.0
Content-Language: en

<!DOCTYPE html>
<html>
<head>
	<title>{}</title>
	<meta name="author" content="Amanda Graven">
	<meta name="description" content="Personal homepage of Amanda Graven">

	<meta charset="UTF-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<link rel="stylesheet" type="text/css" href="/style.css">
	<script src="https://www.google.com/recaptcha/api.js"></script>
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
		</div>"##, title)
}

pub const DOCUMENT_FOOTER: &'static str = "\t</main>\n</body>\n</html>";

pub const SUBMIT_CONTENT: &'static str = r##"
		<article>
			<h1>Submit post</h1>
			<form action="submit.cgi" method="post">
				Title:<br>
				<input type="text" name="title">
				<br>Category:<br>
				<input type="text" name="category">
				<p>Content:<br>
				<textarea rows="1" cols="1" name="content"></textarea>

				<p>User:<br>
				<input type="text" name="user">
				<br>Password:<br>
				<input type="password" name="password">

				<p>
				<input id="submit" type="submit" value="Submit"/>
				<div class="g-recaptcha" data-sitekey="6LdO2SoUAAAAAPOph0HIJ7mUUEnDsG_mfS0AHL1L"></div>
			</form>
		</article>"##;
