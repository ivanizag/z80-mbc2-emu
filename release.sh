cargo build --release
mkdir -p release/z80-mbc2-emu-for-linux
cp README.md download.sh target/release/z80-mbc2-emu release/z80-mbc2-emu-for-linux/
zip -r release/z80-mbc2-emu-for-linux.zip release/z80-mbc2-emu-for-linux

cargo build --release --target x86_64-pc-windows-gnu
mkdir -p release/z80-mbc2-emu-for-windows
cp README.md download.bat target/x86_64-pc-windows-gnu/release/z80-mbc2-emu.exe release/z80-mbc2-emu-for-windows/
zip -r release/z80-mbc2-emu-for-windows.zip release/z80-mbc2-emu-for-windows

