# Markdown
This is a simple markdown with a single request

````http
GET {{test_host}}/no-body
Accept: application/json

SNAPSHOT
status: 200

content-length: 17
content-type: application/json
date: {{_:timestamp("%a, %d %b %Y %H:%M:%S %Z"):"Sat, 10 May 2025 11:23:24 GMT"}}

{
  "hello": "world"
}
````

There is also some text after. Which should be kept.