extern crate chrono;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

use std::fs::{File, create_dir_all, metadata, rename};
use std::io::Write;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use chrono::{Datelike, NaiveDateTime};

mod podcast;
mod list;
mod lib;
mod enclosures;
mod cli;

use podcast::*;
use lib::*;
use enclosures::get_enclosures;

fn main() {
    let args = cli::build_cli().get_matches();
    let podcast_list =
        list::List::read(String::from(
            args.value_of("list").expect("Error parsing podcast list"),
        ));
    let cache_list =
        file_to_string(args.value_of("db").expect("Error parsing cache list:"));
    if args.is_present("pretend") {
        println!("Only pretending to download, marking chapters as read");
    }
    println!(
        "Downloading to: {}",
        podcast_list.target_path.to_str().unwrap()
    );
    for podcast in podcast_list.podcasts {
        println!("{}", podcast.name);
        let feed = match reqwest::get(&podcast.url) {
            Ok(mut content) => {
                match content.text() {
                    Ok(content_text) => content_text,
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
                        let mut local_file_path =
                            PathBuf::from(&podcast_list.target_path);
                        let mut final_file_path =
                            PathBuf::from(&podcast_list.target_path);
                        PathBuf::from(&podcast_list.target_path);
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
                    append_string_to_file(
                        &args.value_of("db").expect("Error parsing cache list:"),
                        &file_url,
                    );
                    append_string_to_file(
                        &args.value_of("db").expect("Error parsing cache list:"),
                        &String::from("\n"),
                    );
                }
                _ => continue,
            };
        }
    }
}
