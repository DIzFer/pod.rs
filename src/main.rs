extern crate futures;
extern crate hyper;
extern crate tokio_core;

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;

struct Podcast<'a> {
    name: &'a str,
    tempo: f32,
    url: hyper::Uri,
}

impl<'a> Podcast<'a> {
    pub fn new(name: &'a str, tempo: f32, url_pre: &'a str) -> Podcast<'a> {
        let url: hyper::Uri = match url_pre.parse(){
            Ok(uri) => uri,
            Err(_) => panic!("Podcast {} doesn't appear to have a valid URL", name),
        };
        Podcast {
            name,
            tempo,
            url,
        }
    }
}

impl<'a> std::fmt::Display for Podcast<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Name: {}\nTempo: {}\nURL: {}", self.name, self.tempo, self.url)
    }
}

fn read_podcast_list(file: &String) -> String {
    let mut file = File::open(file).expect("No such file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Couldn't read file contents");
    contents
}

fn reverse_words(string: String) -> String {
    let string_iter = string.split_whitespace().rev();
    let mut reversed_string = String::new();
    for word in string_iter {
        reversed_string.push_str(word);
        reversed_string.push(' ');
    };
    reversed_string
}
/*
fn get_feed(url: hyper::Uri) {
    let mut core = Core::new().expect("Failed to set up Core");
    let client = Client::new(&core.handle());
    let mut feed = String::new;
    let res = client.get(url).send().unwrap().read_to_string(&mut feed).unwrap();
    println!("{}", feed);
}*/

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => panic!("Error: missing argument: path to list file"),
        2 => {
            let podcast_list = read_podcast_list(&args[1]);
            for podcast in podcast_list.lines() {
                let mut podcast = podcast.split_whitespace().rev();
                let url = podcast.next().unwrap();
                let tempo: f32;
                let name_or_tempo = String::from(podcast.next().unwrap());
                let name_reversed;
                match name_or_tempo.parse::<f32>() {
                    Ok(float) => {
                        tempo = float;
                        let mut name_unwrapped = String::from(podcast.next().unwrap());
                        for fragment in podcast {
                            name_unwrapped.push(' ');
                            name_unwrapped.push_str(fragment);
                        }
                        name_reversed = Some(name_unwrapped);
                    },
                    Err(_) => {
                        match podcast.next() {
                            Some(string) => {
                                let mut name_unwrapped = String::from(name_or_tempo);
                                name_unwrapped.push(' ');
                                name_unwrapped.push_str(string);
                                for fragment in podcast {
                                    name_unwrapped.push(' ');
                                    name_unwrapped.push_str(fragment);
                                }
                                name_reversed = Some(name_unwrapped);
                            },
                            None => name_reversed = Some(name_or_tempo),
                        };
                        tempo = 1.17;
                    },
                };
                let name = reverse_words(name_reversed.unwrap());
                let podcast = Podcast::new(name.trim(), tempo, url);
                println!("{}", podcast);
            }
        },
        _ => panic!("Error: too many arguments"),
    }
}
