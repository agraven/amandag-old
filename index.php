<!DOCTYPE html>
<html>
<head>
	<title>Amanda Graven's Homepage</title>
	<meta name="author" content="Amanda Graven">
	<meta name="description" content="Personal homepage of Amanda Graven">

	<meta charset="UTF-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<link rel="stylesheet" type="text/css" href="style.css">
</head>
<body>
	<div id="headbar">
		<div id="headbar-content">
			<div class="title" style="float: left;">Amanda's terrible homepage</div>
			<ul class="navbar">
				<li>Home</li>
				<li><a href="/about">About</a></li>
			</ul>
		</div>
	</div>
	<div id="easter-egg"> </div>
	<main>
		<div id="body-title">
			<div class="title">Amanda's terrible homepage</div>
		</div>
		<?php
			$host = "localhost";
			$user = "readonly";
			$password = "1234";
			$database = "amandag";

			$connection = new mysqli($host, $user, $password, $database);

			if ($connection->connect_error) {
				die("<article><h1>Critical error!</h1>Connecting to database failed: " .
					mysqli_connect_error() .
					"</article>");
			}

			$query = "SELECT title, content, category, post_time, edit_time FROM posts ORDER BY id DESC LIMIT 15";
			$result = $connection->query($query);

			if ($result->num_rows > 0) {
				while ($row = $result->fetch_assoc()) {
					echo "<article>".
						"<h1>".$row['title']."</h1>".
						"<header>Submitted ".$row['post_time'];
						if ($row['edit_time'] > $row['post_time']) { echo ", last edited".$row['edit_time']; }
						echo "</header><header class=\"right\">Filed under ".$row['category']."</header>\n<p>".
						$row['content'].
						"</article>";
				}
			} else {
				echo "<article><h1>No articles were found</h1>Either none have been submitted in this category yet, or an error occured.</article>";
			}
		?>
		</div>
	</main>
</body>
</html>
