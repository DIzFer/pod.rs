extern crate hyper;
extern crate reqwest;
extern crate sxd_document;
extern crate sxd_xpath;

use sxd_document::parser;
use sxd_xpath::evaluate_xpath;

use std::env;
use std::fs::File;
use std::io::Read;
use hyper::Uri;

struct Podcast<'a> {
    name: &'a str,
    tempo: f32,
    url: hyper::Uri,
}

impl<'a> Podcast<'a> {
    pub fn new(name: &'a str, tempo: f32, url_pre: &'a str) -> Podcast<'a> {
        let url: Uri = match url_pre.parse(){
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

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => panic!("Error: missing argument: path to list file"),
        2 => {
            let podcast_list = read_podcast_list(&args[1]);
            let mut podcast_list_iter = podcast_list.lines();
            let mut config = podcast_list_iter.next().unwrap().split_whitespace().rev();
            let default_tempo = config.next().expect("No default tempo configured");
            let mut target_dir_reversed = String::new();
            for string in config {
                target_dir_reversed.push(' ');
                target_dir_reversed.push_str(string);
            };
            let target_dir = reverse_words(target_dir_reversed);
            let target_dir = target_dir.trim();
            println!("{}, {}", target_dir, default_tempo);
            for podcast in podcast_list_iter {
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
                        tempo = default_tempo.parse().unwrap();
                    },
                };
                let name = reverse_words(name_reversed.unwrap());
                let podcast = Podcast::new(name.trim(), tempo, url);
                println!("{}", podcast.name);
                let feed = reqwest::get(url).unwrap().text().unwrap();
                let feed_parsed = parser::parse(&feed).expect("Unable to parse XML data");
                let feed_document = feed_parsed.as_document();
                for item in vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
                    let realurl = evaluate_xpath(&feed_document,
                                                 &format!("rss/channel/item[{}]/enclosure/@url", item)
                                                  ).expect("Unable to parse XML data").string();
                    let basename = realurl.split("/").last().unwrap();
                    let basename = basename.rsplit("?").last().unwrap();
                    println!("└─ Downloading {}", basename);
                };
            }
        },
        _ => panic!("Error: too many arguments"),
    }
}
