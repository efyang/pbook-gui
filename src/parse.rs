use download::Download;

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
    for entry in vec_data {
        
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
        .connect("")
}

fn remove_blanks(vec_data: Vec<String>) -> Vec<String> {
    vec_data.iter()
        .filter(|&s| s != &"".to_string())
        .map(|s| s.clone())
        .collect::<Vec<String>>()
}

struct Category {
    name: String,
    subcategories: Vec<String>,
}

// removes the identifiers (# in this case) from the category title
fn get_title_name(title_identifier: char, raw_name: String) -> String {
    raw_name.chars()
        .filter(|&c| c != title_identifier)
        .collect::<String>()
}

//returns (name, url)
fn get_item_info(raw_item: String) -> (String, String) {
    unimplemented!();
}
