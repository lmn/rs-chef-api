use chrono::*;
use http_headers::*;
use hyper::header::Headers;
use openssl::hash::{MessageDigest, hash2};
use openssl::rsa::Rsa;
use openssl::rsa::PKCS1_PADDING;
use rustc_serialize::base64::ToBase64;
use std::fmt;
use std::fs::File;
use std::io::Read;
use utils::{expand_string, squeeze_path};
use authentication::BASE64_AUTH;
use errors::*;
use std::ascii::AsciiExt;

pub struct Auth11 {
    #[allow(dead_code)] api_version: String,
    body: Option<String>,
    date: String,
    keypath: String,
    method: String,
    path: String,
    userid: String,
}

impl fmt::Debug for Auth11 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Auth11")
            .field("method", &self.method)
            .field("userid", &self.userid)
            .field("path", &self.path)
            .field("body", &self.body)
            .field("keypath", &self.keypath)
            .finish()
    }
}

impl Auth11 {
    pub fn new(
        path: &str,
        key: &str,
        method: &str,
        userid: &str,
        api_version: &str,
        body: Option<String>,
    ) -> Auth11 {
        let dt = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let userid: String = userid.into();
        let method = String::from(method).to_ascii_uppercase();

        Auth11 {
            api_version: api_version.into(),
            body: body,
            date: dt,
            keypath: key.into(),
            method: method.into(),
            path: squeeze_path(path.into()),
            userid: userid,
        }
    }

    fn hashed_path(&self) -> Result<String> {
        debug!("Path is: {:?}", &self.path);
        let hash = hash2(MessageDigest::sha1(), &self.path.as_bytes())?.to_base64(BASE64_AUTH);
        Ok(hash)
    }

    fn content_hash(&self) -> Result<String> {
        let body = expand_string(&self.body);
        let content = hash2(MessageDigest::sha1(), body.as_bytes())?.to_base64(BASE64_AUTH);
        debug!("{:?}", content);
        Ok(content)
    }

    fn canonical_user_id(&self) -> Result<String> {
        hash2(MessageDigest::sha1(), &self.userid.as_bytes())
            .and_then(|res| Ok(res.to_base64(BASE64_AUTH)))
            .map_err(|res| res.into())
    }

    fn canonical_request(&self) -> Result<String> {
        let cr = format!(
            "Method:{}\nHashed Path:{}\n\
             X-Ops-Content-Hash:{}\n\
             X-Ops-Timestamp:{}\nX-Ops-UserId:{}",
            &self.method,
            try!(self.hashed_path()),
            try!(self.content_hash()),
            self.date,
            try!(self.canonical_user_id())
        );
        debug!("Canonical Request is: {:?}", cr);
        Ok(cr)
    }

    fn encrypted_request(&self) -> Result<String> {
        let mut key: Vec<u8> = vec![];
        match File::open(&self.keypath) {
            Ok(mut fh) => {
                try!(fh.read_to_end(&mut key));
                let key = try!(Rsa::private_key_from_pem(key.as_slice()));

                let cr = try!(self.canonical_request());
                let cr = cr.as_bytes();

                let mut hash: Vec<u8> = vec![0; key.size()];
                try!(key.private_encrypt(cr, &mut hash, PKCS1_PADDING));
                Ok(hash.to_base64(BASE64_AUTH))
            }
            Err(_) => Err(ErrorKind::PrivateKeyError(self.keypath.clone()).into()),
        }
    }

    pub fn build(self, headers: &mut Headers) -> Result<()> {
        let hsh = try!(self.content_hash());
        headers.set(OpsContentHash(hsh));

        headers.set(OpsSign(String::from("algorithm=sha1;version=1.1")));
        headers.set(OpsTimestamp(self.date.clone()));
        headers.set(OpsUserId(self.userid.clone()));

        let enc = try!(self.encrypted_request());
        let mut i = 1;
        for h in enc.split('\n') {
            let key = format!("X-Ops-Authorization-{}", i);
            headers.set_raw(key, vec![h.as_bytes().to_vec()]);
            i += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Auth11;

    const PATH: &'static str = "/organizations/clownco";
    const BODY: &'static str = "Spec Body";
    const USER: &'static str = "spec-user";
    const DT: &'static str = "2009-01-01T12:00:00Z";

    const PRIVATE_KEY: &'static str = "fixtures/spec-user.pem";

    #[test]
    fn test_canonical_user_id() {
        let auth = Auth11 {
            api_version: String::from("1"),
            body: Some(String::from(BODY)),
            date: String::from(DT),
            keypath: String::from(PRIVATE_KEY),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
        };
        assert_eq!(
            auth.canonical_user_id().unwrap(),
            "EAF7Wv/hbAudWV5ZkwKz40Z/lO0="
        )
    }

    #[test]
    fn test_canonical_request() {
        let auth = Auth11 {
            api_version: String::from("1"),
            body: Some(String::from(BODY)),
            date: String::from(DT),
            keypath: String::from(""),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
        };
        assert_eq!(
            auth.canonical_request().unwrap(),
            "Method:POST\nHashed \
             Path:YtBWDn1blGGuFIuKksdwXzHU9oE=\nX-Ops-Content-Hash:\
             DFteJZPVv6WKdQmMqZUQUumUyRs=\nX-Ops-Timestamp:2009-01-01T12:00:\
             00Z\nX-Ops-UserId:EAF7Wv/hbAudWV5ZkwKz40Z/lO0="
        )
    }

    #[test]
    fn test_private_key() {
        let auth = Auth11 {
            api_version: String::from("1"),
            body: Some(String::from(BODY)),
            date: String::from(DT),
            keypath: String::from(PRIVATE_KEY),
            method: String::from("POST"),
            path: String::from(PATH),
            userid: String::from(USER),
        };
        assert_eq!(
            &auth.encrypted_request().unwrap(),
            "UfZD9dRz6rFu6LbP5Mo1oNHcWYxpNIcUfFCffJS1FQa0GtfU/vkt3/O5HuCM\n\
             1wIFl/U0f5faH9EWpXWY5NwKR031Myxcabw4t4ZLO69CIh/3qx1XnjcZvt2w\n\
             c2R9bx/43IWA/r8w8Q6decuu0f6ZlNheJeJhaYPI8piX/aH+uHBH8zTACZu8\n\
             vMnl5MF3/OIlsZc8cemq6eKYstp8a8KYq9OmkB5IXIX6qVMJHA6fRvQEB/7j\n\
             281Q7oI/O+lE8AmVyBbwruPb7Mp6s4839eYiOdjbDwFjYtbS3XgAjrHlaD7W\n\
             FDlbAG7H8Dmvo+wBxmtNkszhzbBnEYtuwQqT8nM/8A=="
        )
    }

}
