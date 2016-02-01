pub use data::*;
use std::ascii::AsciiExt;

// pub fn parse(data: &str) -> Vec<Vec<Download>> {
pub fn parse(data: &str) -> Vec<Category> {
    let padded_data = blanks_to_newlines(data.split('\n')
                                             .map(|l| l.trim().to_string())
                                             .collect::<Vec<String>>());
    let category_data = padded_data.split("\n")
                                   .map(|s| s.to_string())
                                   .collect::<Vec<String>>();
    get_categories(remove_blanks(category_data), '#')
}

pub fn get_categories(vec_data: Vec<String>, title_identifier: char) -> Vec<Category> {
    let mut categories: Vec<Category> = Vec::with_capacity(vec_data.len());
    let mut category_name: String = "Index".to_string();
    let mut category: Category = Category::new(category_name, vec![]);
    for entry in vec_data {
        let title_head = [title_identifier; 3]
                             .into_iter()
                             .map(|&c| c.clone())
                             .collect::<String>() + " ";

        if entry.contains(&title_head) {
            categories.push(category.clone());
            category_name = get_title_name('#', entry);
            category = Category::new(category_name, vec![])
        } else {
            match get_item_info(entry) {
                Some(data) => {
                    // data.1 is url
                    if data.1.to_ascii_lowercase().contains("pdf") {
                        let dl = Download::new(&data.0, &data.1);
                        category.add_download(dl);
                    }
                }
                None => {}
            }
        }
    }

    // remove all unnecessary categories
    categories = categories.iter()
                           .filter(|c| {
                               !c.get_name().to_ascii_lowercase().contains("index") &&
                               c.get_downloads().len() != 0
                           })
                           .map(|c| c.clone())
                           .collect();
    categories
}

fn blanks_to_newlines(vec_data: Vec<String>) -> String {
    vec_data.iter()
            .map(|s| {
                if s == &"".to_string() {
                    "\n".to_string()
                } else {
                    (s.clone() + "\n")
                }
            })
            .collect::<Vec<String>>()
            .join("")
}

fn remove_blanks(vec_data: Vec<String>) -> Vec<String> {
    vec_data.iter()
            .filter(|&s| s != &"".to_string())
            .map(|s| s.clone())
            .collect::<Vec<String>>()
}

// removes the identifiers (# in this case) from the category title
fn get_title_name(title_identifier: char, raw_name: String) -> String {
    raw_name.chars()
            .filter(|&c| c != title_identifier)
            .collect::<String>()
}

// returns (name, url)
pub fn get_item_info(raw_item: String) -> Option<(String, String)> {
    let title_start: usize;
    let title_end: usize;
    let title: &str;
    let url_end: usize;
    let url: &str;

    if let Some(start) = raw_item.find("[") {
        title_start = start + 1;
    } else {
        return None;
    }
    if let Some(end) = raw_item.find("]") {
        title_end = end;
    } else {
        return None;
    }
    title = &raw_item[title_start..title_end];

    if let Some(end) = (&raw_item[(title_end + 2)..].to_string()).find(")") {
        url_end = end;
    } else {
        return None;
    }

    url = &raw_item[(title_end + 2)..][..url_end];

    Some((title.to_string(), url.to_string()))
}
