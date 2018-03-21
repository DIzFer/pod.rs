extern crate chrono;
extern crate reqwest;
extern crate sxd_document;
extern crate sxd_xpath;

use std::env;
use std::fs::{File, create_dir_all, metadata, rename, remove_file};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
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
        3 => {
            let podcast_list = file_to_string(&args[1]);
            let cache_list = file_to_string(&args[2]);
            let mut podcast_list_iter = podcast_list.lines();
            let mut config = podcast_list_iter.next().unwrap().split_whitespace().rev();
            let default_tempo = config.next().expect("No default tempo configured");
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
                    }
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
                            }
                            None => name_reversed = Some(name_or_tempo),
                        };
                        tempo = default_tempo.parse().unwrap();
                    }
                };
                let name = reverse_words(name_reversed.unwrap());
                let podcast = Podcast::new(name.trim(), tempo, url);
                println!("{}", podcast.name);
                let feed = reqwest::get(url).unwrap().text().unwrap();
                let feed_parsed = parser::parse(&feed).expect("Unable to parse XML data");
                let feed_document = feed_parsed.as_document();
                let mut urls_to_download = Vec::new();
                let mut item_count = 1;
                loop {
                    match evaluate_xpath(
                        &feed_document,
                        &format!("rss/channel/item[{}]/enclosure/@url", item_count),
                    ) {
                        Ok(value) => {
                            if &value.string() == "" {
                                break;
                            } else {
                                item_count = item_count + 1;
                                let string = value.into_string();
                                urls_to_download.push(string);
                            };
                        }
                        Err(_) => println!("Something really weird happened here..."),
                    };
                }
                for file_url in urls_to_download {
                    match cache_list.rfind(&file_url) {
                        None => {
                            let basename = file_url.split("/").last().unwrap();
                            let basename = basename.rsplit("?").last().unwrap();
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
                            Command::new("ffmpeg")
                                .args(
                                    &[
                                        "-i",
                                        local_file_path.to_str().unwrap(),
                                        "-vcodec",
                                        "copy",
                                        "-an",
                                        cover_file_path.to_str().unwrap(),
                                    ],
                                )
                                .output()
                                .expect("FFMPEG failed");
                            if &podcast.tempo == &(1.0 as f32) {
                                rename(local_file_path, final_file_path).expect(
                                    "Could not rename file",
                                );
                            } else {
                                final_file_path.set_extension("opus");
                                Command::new("ffmpeg")
                                    .args(
                                        &[
                                            "-i",
                                            local_file_path.to_str().unwrap(),
                                            "-filter:a",
                                            &format!("atempo={}", &podcast.tempo),
                                            "-b:a",
                                            "192k",
                                            final_file_path.to_str().unwrap(),
                                        ],
                                    )
                                    .output()
                                    .expect("FFMPEG failed");
                                remove_file(&local_file_path).expect("Could not remove file");
                            };
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
