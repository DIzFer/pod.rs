extern crate serde;
extern crate serde_xml_rs;

#[derive(Debug, Deserialize)]
struct RSS {
    pub channel: Channel,
}

#[derive(Debug, Deserialize)]
struct Channel {
    pub item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    pub enclosure: Enclosure,
}

#[derive(Debug, Deserialize)]
struct Enclosure {
    pub url: String,
}

pub fn get_enclosures(document: String) -> Vec<String> {
    let mut list_of_urls = Vec::new();
    let deserialized_feed: RSS = serde_xml_rs::deserialize(document.as_bytes())
        .expect("Not a valid XML document");
    for item in deserialized_feed.channel.item {
        list_of_urls.push(item.enclosure.url);
    }
    list_of_urls
}
