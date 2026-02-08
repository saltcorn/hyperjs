pub fn create_html_document(title: String, body: String) -> String {
  format!(
    r#"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title> {title} </title>
</head>
<body>
<pre> {body} </pre>
</body>
</html>\n'
"#
  )
}
