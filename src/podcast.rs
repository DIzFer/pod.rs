extern crate hyper;

use std;

pub struct Podcast {
    pub name: String,
    pub url: String,
}

impl std::fmt::Display for Podcast {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Name: {}\nURL: {}", self.name, self.url)
    }
}
