extern crate chrono;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

use std::env;
use std::fs::{File, create_dir_all, metadata, rename};
use std::io::Write;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use chrono::{Datelike, NaiveDateTime};

mod podcast;
mod lib;
mod enclosures;
mod cli;

use podcast::*;
use lib::*;
use enclosures::get_enclosures;

fn main() {
    let args = cli::build_cli().get_matches();
    let podcast_list = file_to_string(&args.value_of("list").expect(
        "Error parsing podcast list:",
    ));
    let cache_list =
        file_to_string(args.value_of("db").expect("Error parsing cache list:"));
    if args.is_present("pretend") {
        println!("Only pretending to download, marking chapters as read");
    }
    let mut podcast_list_iter = podcast_list.lines();
    let config = podcast_list_iter.next().unwrap().split_whitespace().rev();
    let mut target_dir_reversed = String::new();
    for string in config {
        target_dir_reversed.push(' ');
        target_dir_reversed.push_str(string);
    }
    let mut target_dir = reverse_words(target_dir_reversed);
    if target_dir.starts_with('~') {
        target_dir = target_dir.replacen(
            "~",
            env::home_dir().unwrap().to_str().unwrap(),
            1,
        );
    }
    println!("{}", target_dir);
    let target_dir = PathBuf::from(target_dir.trim());
    for podcast in podcast_list_iter {
        let mut podcast = podcast.split_whitespace().rev();
        let url = podcast.next().unwrap();
        let name = reverse_words(podcast.collect::<Vec<&str>>().join(" "));
        let podcast = Podcast::new(name.trim(), url);
        println!("{}", podcast.name);
        let feed = match reqwest::get(url) {
            Ok(mut content) => {
                match content.text() {
                    Ok(mut content_text) => content_text,
                    Err(error) => {
                        println!("└─ {}", error);
                        continue;
                    }
                }
            }
            Err(error) => {
                println!("└─ {}", error);
                continue;
            }
        };
        let urls_to_download: Vec<String> = get_enclosures(feed);
        for file_url in urls_to_download {
            match cache_list.rfind(&file_url) {
                None => {
                    let basename = file_url.split("/").last().unwrap();
                    let basename = basename.rsplit("?").last().unwrap();
                    if args.is_present("pretend") {
                        println!("└─ Marking {} as fetched", basename);
                    } else {
                        println!("└─ Downloading {}", basename);
                        let mut remote_file = match reqwest::get(&file_url) {
                            Ok(mut content) => content,
                            Err(error) => {
                                println!("└─ {}", error);
                                continue;
                            }
                        };
                        let mut buffer: Vec<u8> = vec![];
                        remote_file.copy_to(&mut buffer).unwrap();
                        let mut local_file_path = PathBuf::from(&target_dir);
                        let mut final_file_path = PathBuf::from(&target_dir);
                        PathBuf::from(&target_dir);
                        local_file_path.push(&podcast.name);
                        final_file_path.push(&podcast.name);
                        create_dir_all(&local_file_path).expect(
                            "Could not create necessary directories",
                        );
                        local_file_path.push(&basename);
                        let mut local_file =
                            File::create(&local_file_path).expect(
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
                        let file_date =
                            NaiveDateTime::from_timestamp(file_date as i64, 0);
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
                    append_string_to_file(&cache_list, &file_url);
                    append_string_to_file(&cache_list, &String::from("\n"));
                }
                _ => continue,
            };
        }
    }
}
