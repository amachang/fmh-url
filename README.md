# Forward Matching Hierarchical URL

Convert url to hierarchical format string could be reversed to url.

It's useful with database indexing and searching.

## Example

`https://sub.example.com/users/profile?b=123&a=321#section1` -> `com.example.sub/https/443///users/profile?b=123&a=321#section1`
`ftp://user:password@example.com/file.txt` -> `com.example/ftp/21/user:password//file.txt`
`http://127.0.0.1:8080/index.html` -> `127.0.0.1/http/8080///index.html`
`mailto:example@example.com` -> `/maito////example@example.com`
`https://[::1]/index.html` -> `[0000:0000:0000:0000:0000:0000:0000:0001]/https/443///index.html`

