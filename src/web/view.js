var captchaLoad = function() {
	grecaptcha.render(
		'root-captcha',
		{ 'sitekey' : '6Lcs3ywUAAAAAN7ASI4sa9wr8ujsfoZd0xgnpnWV' }
	);
}
function setHeight(elem) {
	elem.style.height = '1px';
	elem.style.height = elem.scrollHeight + 5 + 'px';
}
function show(element) {
	element.style.display = "block";
	grecaptcha.render(
		element.getElementsByClassName("g-recaptcha")[0],
		{ 'sitekey' : '6Lcs3ywUAAAAAN7ASI4sa9wr8ujsfoZd0xgnpnWV' }
	);
}
function hide(element) {
	element.style.display = "none";
}
function send(form) {
	function urlencodeFormData(fd) {
		var s = '';
		function encode(s) {
			return encodeURIComponent(s).replace(/%20/g,'+');
		}
		for(var pair of fd.entries()) {
			if(typeof pair[1] == 'string') {
				s += (s?'&':'') + encode(pair[0])+'='+encode(pair[1]);
			}
		}
		return s;
	}
	var data = new FormData(form);
	data.set("id", id);
	for (var key of data.keys()) {
		console.log('Key: ' + key);
	}

	var request = new XMLHttpRequest();
	request.open("POST", "/comment", true);
	request.setRequestHeader("Content-Type", "application/x-www-form-urlencoded");
	request.onreadystatechange = function() {
		if (this.readyState == 4) {
			if (this.status == 200) {
				form.parentElement.innerHTML += this.responseText;
			} else {
				alert("Error " + this.status + ": " + this.statusText + ": " + this.responseText);
			}
		}
	}
	request.send(urlencodeFormData(data));
}
