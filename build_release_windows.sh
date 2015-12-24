cargo build --release --features "all-themes"
cargo rustc --release --features "all-themes" --bin pbook-gui -- "-Clink-args=-mwindows"
cargo build --release --features "all-themes"
