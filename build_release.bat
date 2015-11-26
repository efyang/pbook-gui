cargo rustc --release -- "-Clink-args=-mwindows"
cscript zip.vbs "target/release/pbook-gui.zip" "target/release/pbook-gui.exe" "target/release/pbook-gui.exe.manifest" "target/release/iup.dll"
