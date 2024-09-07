use std::net;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid FMH-URL: {0}")]
    InvalidFmhUrl(String),
}

pub fn convert(url: &Url) -> String {
    let mut fmh_url = String::new();
    if let Some(host) = url.host() {
        fmh_url.push_str(&convert_host(&host));
        log::trace!("host added: {}", fmh_url);
    }
    fmh_url.push('/');

    let scheme = url.scheme();
    fmh_url.push_str(scheme);
    fmh_url.push('/');
    log::trace!("scheme added: {}", fmh_url);

    let port = url.port_or_known_default();
    if let Some(port) = port {
        fmh_url.push_str(port.to_string().as_str());
        log::trace!("port added: {}", fmh_url);
    }
    fmh_url.push('/');

    match (url.username(), url.password()) {
        ("", None) => {},
        (username, password) => {
            fmh_url.push_str(username);
            log::trace!("username added: {}", fmh_url);
            if let Some(password) = password {
                fmh_url.push(':');
                fmh_url.push_str(password);
            }
            log::trace!("password added: {}", fmh_url);
        },
    }

    fmh_url.push('/');
    fmh_url.push_str(url.path());
    log::trace!("path added: {}", fmh_url);

    let query = url.query();
    if let Some(query) = query {
        fmh_url.push_str(&format!("?{}", query));
        log::trace!("query added: {}", fmh_url);
    }

    if let Some(fragment) = url.fragment() {
        fmh_url.push_str(&format!("#{}", fragment));
        log::trace!("fragment added: {}", fmh_url);
    }

    fmh_url
}

pub fn revert(fmh_url: impl AsRef<str>) -> Result<Url, Error> {
    let fmh_url = fmh_url.as_ref();
    let mut parts = fmh_url.splitn(5, '/').collect::<Vec<_>>();
    if parts.len() != 5 {
        return Err(Error::InvalidFmhUrl(fmh_url.to_string()));
    }
    let path = parts.pop().expect("checked");
    let username_password = parts.pop().expect("checked");
    let port = parts.pop().expect("checked");
    let scheme = parts.pop().expect("checked");
    let host = parts.pop().expect("checked");
    
    let mut url = String::new();
    url.push_str(scheme);
    url.push(':');
    // has authority
    if !host.is_empty() || !port.is_empty() || !username_password.is_empty() {
        url.push_str("//");
    }
    if !username_password.is_empty() {
        url.push_str(username_password);
        url.push('@');
    }
    url.push_str(&revert_host(host)?);
    if !port.is_empty() {
        url.push(':');
        url.push_str(port);
    }
    url.push_str(path);

    log::trace!("reverted URL: {}", url);

    Ok(Url::parse(&url).expect(&format!("should be valid url, but it's a bug: {}", url)))
}

fn convert_host(host: &url::Host<impl AsRef<str>>) -> String {
    match host {
        url::Host::Domain(domain) => {
            let domain = domain.as_ref();
            domain.split('.').rev().collect::<Vec<_>>().join(".")
        },
        url::Host::Ipv4(ip) => ip.to_string(),
        url::Host::Ipv6(ip) => format!("[{}]", expand_ipv6(ip)),
    }
}

fn revert_host(host: impl AsRef<str>) -> Result<String, Error> {
    let host = host.as_ref();
    if host.starts_with('[') && host.ends_with(']') {
        Ok(host.to_string())
    } else if let Ok(ipv4) = host.parse::<net::Ipv4Addr>() {
        Ok(ipv4.to_string())
    } else {
        Ok(host.split('.').rev().collect::<Vec<_>>().join(".").to_string())
    }
}

fn expand_ipv6(ip: &net::Ipv6Addr) -> String {
    let ip = ip.segments();
    format!("{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}", ip[0], ip[1], ip[2], ip[3], ip[4], ip[5], ip[6], ip[7])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert() {
        let _= env_logger::try_init();

        assert_eq!(convert(&Url::parse("https://sub.example.com/users/profile?b=123&a=321#section1").unwrap()), "com.example.sub/https/443///users/profile?b=123&a=321#section1");
        assert_eq!(convert(&Url::parse("ftp://user:password@example.com/file.txt").unwrap()), "com.example/ftp/21/user:password//file.txt");
        assert_eq!(convert(&Url::parse("http://127.0.0.1:8080/index.html").unwrap()), "127.0.0.1/http/8080///index.html");
        assert_eq!(convert(&Url::parse("https://[::1]/index.html").unwrap()), "[0000:0000:0000:0000:0000:0000:0000:0001]/https/443///index.html");
        assert_eq!(convert(&Url::parse("http://example.com").unwrap()), "com.example/http/80///");
        assert_eq!(convert(&Url::parse("http://my-local-server.local-network:8080").unwrap()), "local-network.my-local-server/http/8080///");
        assert_eq!(convert(&Url::parse("sftp://my-local-server.local-network/").unwrap()), "local-network.my-local-server/sftp////");
        assert_eq!(convert(&Url::parse("mailto:example@example.com").unwrap()), "/mailto///example@example.com");
        assert_eq!(convert(&Url::parse("file:///tmp/foo").unwrap()), "/file////tmp/foo");
        assert_eq!(convert(&Url::parse("blob:https://example.com/foo").unwrap()), "/blob///https://example.com/foo");
        assert_eq!(convert(&Url::parse("https://example.com").unwrap()), "com.example/https/443///");
        assert_eq!(convert(&Url::parse("https://example.com/").unwrap()), "com.example/https/443///");
        assert_eq!(convert(&Url::parse("data:text/plain,Stuff").unwrap()), "/data///text/plain,Stuff");
        assert_eq!(convert(&Url::parse("https://user@example.com/").unwrap()), "com.example/https/443/user//");
        assert_eq!(convert(&Url::parse("https://:password@example.com/").unwrap()), "com.example/https/443/:password//");
        assert_eq!(convert(&Url::parse("https://@example.com/").unwrap()), "com.example/https/443///");
        assert_eq!(convert(&Url::parse("ftp://rms@example.com/").unwrap()), "com.example/ftp/21/rms//");
        assert_eq!(convert(&Url::parse("https://example.com/?a=%E3%81%82").unwrap()), "com.example/https/443///?a=%E3%81%82");
        assert_eq!(convert(&Url::parse("https://xn--l8j/%E3%81%84?%E3%81%86=%E3%81%88#%E3%81%8A").unwrap()), "xn--l8j/https/443///%E3%81%84?%E3%81%86=%E3%81%88#%E3%81%8A");
        assert_eq!(convert(&Url::parse("https://あ/い?う=え#お").unwrap()), "xn--l8j/https/443///%E3%81%84?%E3%81%86=%E3%81%88#%E3%81%8A");
    }

    #[test]
    fn test_revert() {
        let _= env_logger::try_init();

        assert_eq!(revert("com.example.sub/https/443///users/profile?b=123&a=321#section1").unwrap(), Url::parse("https://sub.example.com/users/profile?b=123&a=321#section1").unwrap());
        assert_eq!(revert("com.example/ftp/21/user:password//file.txt").unwrap(), Url::parse("ftp://user:password@example.com/file.txt").unwrap());
        assert_eq!(revert("127.0.0.1/http/8080///index.html").unwrap(), Url::parse("http://127.0.0.1:8080/index.html").unwrap());
        assert_eq!(revert("[0000:0000:0000:0000:0000:0000:0000:0001]/https/443///index.html").unwrap(), Url::parse("https://[::1]/index.html").unwrap());
        assert_eq!(revert("com.example/http/80///").unwrap(), Url::parse("http://example.com").unwrap());
        assert_eq!(revert("local-network.my-local-server/http/8080///").unwrap(), Url::parse("http://my-local-server.local-network:8080").unwrap());
        assert_eq!(revert("local-network.my-local-server/sftp////").unwrap(), Url::parse("sftp://my-local-server.local-network/").unwrap());
        assert_eq!(revert("/mailto///example@example.com").unwrap(), Url::parse("mailto:example@example.com").unwrap());
        assert_eq!(revert("/file////tmp/foo").unwrap(), Url::parse("file:///tmp/foo").unwrap());
        assert_eq!(revert("/blob///https://example.com/foo").unwrap(), Url::parse("blob:https://example.com/foo").unwrap());
        assert_eq!(revert("com.example/https/443///").unwrap(), Url::parse("https://example.com").unwrap());
        assert_eq!(revert("com.example/https/443///").unwrap(), Url::parse("https://example.com/").unwrap());
        assert_eq!(revert("/data///text/plain,Stuff").unwrap(), Url::parse("data:text/plain,Stuff").unwrap());
        assert_eq!(revert("com.example/https/443/user//").unwrap(), Url::parse("https://user@example.com/").unwrap());
        assert_eq!(revert("com.example/https/443/:password//").unwrap(), Url::parse("https://:password@example.com/").unwrap());
        assert_eq!(revert("com.example/https/443///").unwrap(), Url::parse("https://@example.com/").unwrap());
        assert_eq!(revert("com.example/ftp/21/rms//").unwrap(), Url::parse("ftp://rms@example.com/").unwrap());
        assert_eq!(revert("com.example/https/443///?a=%E3%81%82").unwrap(), Url::parse("https://example.com/?a=%E3%81%82").unwrap());
        assert_eq!(revert("xn--l8j/https/443///%E3%81%84?%E3%81%86=%E3%81%88#%E3%81%8A").unwrap(), Url::parse("https://xn--l8j/%E3%81%84?%E3%81%86=%E3%81%88#%E3%81%8A").unwrap());
        assert_eq!(revert("xn--l8j/https/443///%E3%81%84?%E3%81%86=%E3%81%88#%E3%81%8A").unwrap(), Url::parse("https://あ/い?う=え#お").unwrap());
    }
}
