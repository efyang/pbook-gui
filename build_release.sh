cargo build --release
cd "target/release"
mv "pbook-gui" "pbook-guic"
mkdir "pbook-gui"
cp "pbook-guic" "pbook-gui/pbook-gui"
tar cfJ "pbook-gui.tar.xz" "pbook-gui"
rm -rf "pbook-gui"
mv "pbook-guic" "pbook-gui"
