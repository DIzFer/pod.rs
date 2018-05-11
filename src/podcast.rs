extern crate hyper;

use std;
use self::hyper::Uri;

pub struct Podcast<'a> {
    pub name: &'a str,
    pub url: hyper::Uri,
}

impl<'a> Podcast<'a> {
    pub fn new(name: &'a str, url_pre: &'a str) -> Podcast<'a> {
        let url: Uri = match url_pre.parse() {
            Ok(uri) => uri,
            Err(_) => {
                panic!("Podcast {} doesn't appear to have a valid URL", name)
            }
        };
        Podcast { name, url }
    }
}

impl<'a> std::fmt::Display for Podcast<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Name: {}\nURL: {}", self.name, self.url)
    }
}
