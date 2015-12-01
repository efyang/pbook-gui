#[cfg(unix)]
pub const file_sep: &'static str = "/";
//pub const pbook_raw: &'static str = include_str!("../free-programming-books/free-programming-books.md");

#[cfg(windows)]
pub const file_sep: &'static str = "\\";
//pub const pbook_raw: &'static str = include_str!("..\\free-programming-books\\free-programming-books.md");
