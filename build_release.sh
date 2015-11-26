cargo build --release
cd "target/release"
cp "pbook-gui" "pbook-guic"
tar cfJ "pbook-gui.tar.xz" "pbook-gui"
mv "pbook-guic" "pbook-gui"
