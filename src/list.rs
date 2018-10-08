use std::path::PathBuf;

use Podcast;
use lib;

pub struct List {
    pub target_path: PathBuf,
    pub podcasts: Vec<Podcast>,
}

impl List {
    pub fn read(file: String) -> List {
        let raw_file = lib::file_to_string(&file);
        let mut raw_file_iter = raw_file.lines();
        let target_path = raw_file_iter.next().expect(
            "Error: Your config file is empty!",
        );
        let mut podcast_list: Vec<Podcast> = Vec::new();
        for podcast_line in raw_file_iter {
            let mut podcast_line_iter = podcast_line.split_whitespace().rev();
            let podcast_url = String::from(podcast_line_iter.next().expect(
                "Error: no subscriptions found",
            ));
            let podcast_name =
                lib::reverse_words(
                    podcast_line_iter.collect::<Vec<&str>>().join(" "),
                );
            podcast_list.push(Podcast {
                name: podcast_name,
                url: podcast_url,
            });
        }
        List {
            target_path: PathBuf::from(&lib::replace_tilde(target_path).trim()),
            podcasts: podcast_list,
        }
    }
}
