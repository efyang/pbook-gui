use download::*;

// pub fn parse(data: &str) -> Vec<Vec<Download>> {
pub fn parse(data: &str) -> Vec<String> {
    let padded = blanks_to_newlines(data.split('\n')
                                        .map(|l| l.trim().to_string())
                                        .collect::<Vec<String>>());
    let categories = padded.split("\n")
                           .map(|s| s.to_string())
                           .collect::<Vec<String>>();
    remove_blanks(categories)
}

fn get_categories(vec_data: Vec<String>, title_identifier: char) -> Vec<Category> {
    let mut categories: Vec<Category> = Vec::with_capacity(vec_data.len());
    let mut category_name: String;
    for entry in vec_data {
        if entry.contains("### ") {
            category_name = get_title_name('#', entry);
        }
    }
    unimplemented!();
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
