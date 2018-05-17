use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::env;

pub fn file_to_string(file: &str) -> String {
    let mut file = File::open(file).expect("No such file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect(
        "Couldn't read file contents",
    );
    let mut contents_clean = String::new();
    for line in contents.lines() {
        if !line.starts_with('#') {
            contents_clean.push_str(line);
            contents_clean.push('\n');
        };
    }
    contents_clean
}

pub fn reverse_words(string: String) -> String {
    let string_iter = string.split_whitespace().rev();
    let mut reversed_string = String::new();
    for word in string_iter {
        reversed_string.push_str(word);
        reversed_string.push(' ');
    }
    reversed_string
}

pub fn append_string_to_file(file: &str, string: &String) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .expect("Error trying to open file");
    file.write_all(string.as_bytes()).expect(
        "Unable to write to file",
    );
}

pub fn replace_tilde(path: &str) -> String {
    path.replacen("~", env::home_dir().unwrap().to_str().unwrap(), 1)
}
