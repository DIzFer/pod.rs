extern crate chrono;
extern crate reqwest;
extern crate sxd_document;
extern crate sxd_xpath;

use std::env;
use std::fs::{File, create_dir_all, metadata, rename};
use std::io::Write;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use chrono::{Datelike, NaiveDateTime};
use sxd_document::parser;
use sxd_xpath::evaluate_xpath;

mod podcast;
mod lib;

use podcast::*;
use lib::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 | 2 => println!("Error: missing arguments\nSyntax: `pod.rs podcast.list cache.db`"),
        3 | 4 => {
            let podcast_list = file_to_string(&args[1]);
            let cache_list = file_to_string(&args[2]);
            if args.len() == 4 && &args[3] == "-p" {
                println!("Only pretending to download, marking chapters as read");
            }
            let mut podcast_list_iter = podcast_list.lines();
            let mut config = podcast_list_iter.next().unwrap().split_whitespace().rev();
            let mut target_dir_reversed = String::new();
            for string in config {
                target_dir_reversed.push(' ');
                target_dir_reversed.push_str(string);
            }
            let mut target_dir = reverse_words(target_dir_reversed);
            if target_dir.starts_with('~') {
                target_dir =
                    target_dir.replacen("~", env::home_dir().unwrap().to_str().unwrap(), 1);
            }
            println!("{}", target_dir);
            let target_dir = PathBuf::from(target_dir.trim());
            for podcast in podcast_list_iter {
                let mut podcast = podcast.split_whitespace().rev();
                let url = podcast.next().unwrap();
                let name = String::from(podcast.next().unwrap());
                let name = reverse_words(name_reversed.unwrap());
                let podcast = Podcast::new(name.trim(), url);
                println!("{}", podcast.name);
                let feed = reqwest::get(url).unwrap().text().unwrap();
                let feed_parsed = parser::parse(&feed).expect("Unable to parse XML data");
                let feed_document = feed_parsed.as_document();
                let mut urls_to_download = Vec::new();
                let mut item_count: usize = 1;
                let amount_of_enclosures = &feed.matches("enclosure").count();
                while &item_count <= &amount_of_enclosures {
                    match evaluate_xpath(
                        &feed_document,
                        &format!("rss/channel/item[{}]/enclosure/@url", item_count),
                    ) {
                        Ok(value) => {
                            item_count = item_count + 1;
                            let string = value.into_string();
                            urls_to_download.push(string);
                        }
                        Err(_) => println!("Something really weird happened here..."),
                    };
                }
                for file_url in urls_to_download {
                    match cache_list.rfind(&file_url) {
                        None => {
                            let basename = file_url.split("/").last().unwrap();
                            let basename = basename.rsplit("?").last().unwrap();
                            if args.len() == 4 && &args[3] == "-p" {
                                println!("└─ Marking {} as fetched", basename);
                            } else {
                                println!("└─ Downloading {}", basename);
                                let mut remote_file = reqwest::get(&file_url).expect(&format!(
                                    "Could not download file from {}",
                                    file_url
                                ));
                                let mut buffer: Vec<u8> = vec![];
                                remote_file.copy_to(&mut buffer).unwrap();
                                let mut local_file_path = PathBuf::from(&target_dir);
                                let mut final_file_path = PathBuf::from(&target_dir);
                                let mut cover_file_path = PathBuf::from(&target_dir);
                                local_file_path.push(&podcast.name);
                                final_file_path.push(&podcast.name);
                                cover_file_path.push(&podcast.name);
                                create_dir_all(&local_file_path).expect(
                                    "Could not create necessary directories",
                                );
                                local_file_path.push(&basename);
                                cover_file_path.push("cover.jpg"); //FIXME: Should probably guess extension from actual filetype.
                                //Voice for Android just swallows whatever I put in cover.jpg and displays it fine ¯\_(ツ)_/¯
                                let mut local_file = File::create(&local_file_path).expect(
                                    "Could not create audio file",
                                );
                                local_file.write_all(&buffer).expect(
                                    "Could not write to file",
                                );
                                let file_date = metadata(&local_file_path)
                                    .expect("File wasn't downloaded for some reason")
                                    .modified()
                                    .unwrap()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();
                                let file_date = NaiveDateTime::from_timestamp(file_date as i64, 0);
                                let mut final_file_name = String::from(format!(
                                    "{}-{}-{}-",
                                    file_date.year(),
                                    file_date.month(),
                                    file_date.day()
                                ));
                                final_file_name.push_str(&basename);
                                final_file_path.push(&final_file_name);
                                rename(local_file_path, final_file_path).expect(
                                    "Could not rename file",
                                );
                            }
                            append_string_to_file(&args[2], &file_url);
                            append_string_to_file(&args[2], &String::from("\n"));
                        }
                        _ => continue,
                    };
                }
            }
        }
        _ => println!("Error: too many arguments\nSyntax: `pod.rs podcast.list cache.db`"),
    }
}
