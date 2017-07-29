pub fn format_document_header(title: &str) -> String {
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