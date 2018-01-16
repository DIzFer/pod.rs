use std::fs::File;
use std::io::Read;

pub fn read_podcast_list(file: &String) -> String {
    let mut file = File::open(file).expect("No such file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Couldn't read file contents");
    contents
}

pub fn reverse_words(string: String) -> String {
    let string_iter = string.split_whitespace().rev();
    let mut reversed_string = String::new();
    for word in string_iter {
        reversed_string.push_str(word);
        reversed_string.push(' ');
    };
    reversed_string
}
