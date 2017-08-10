#!/bin/bash
mv target/release/{amandag,index.cgi}
mv target/release/submit{,.cgi}
mv target/release/view{,.cgi}
scp style.css target/release/{index.cgi,submit.cgi,view.cgi} www-data@amandag.net:amandag/htdocs
