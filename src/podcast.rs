extern crate hyper;

use std;
use self::hyper::Uri;

pub struct Podcast<'a> {
    pub name: &'a str,
    pub tempo: f32,
    pub url: hyper::Uri,
}

impl<'a> Podcast<'a> {
    pub fn new(name: &'a str, tempo: f32, url_pre: &'a str) -> Podcast<'a> {
        let url: Uri = match url_pre.parse() {
            Ok(uri) => uri,
            Err(_) => panic!("Podcast {} doesn't appear to have a valid URL", name),
        };
        Podcast { name, tempo, url }
    }
}

impl<'a> std::fmt::Display for Podcast<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Name: {}\nTempo: {}\nURL: {}",
            self.name,
            self.tempo,
            self.url
        )
    }
}
