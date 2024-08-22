# Waffle

Tilemap editor for SNES backgrounds

![waffle](https://github.com/user-attachments/assets/5cb83f3d-8f57-4806-8d05-7fe6353194f8)

# Install

Make sure GTK+ libraries are installed, then go to [Releases](https://github.com/starliteSeeker/waffle/releases) and download the newest version.

Only the linux version is provided because that's the only one I can test.

# Build

~~works on my machine~~ 

## Linux

```sh
#gtk version 4.6, but should(?) work for versions < 4.10
apt install libgtk-4-dev build-essential
#make sure rustup is installed, then
cargo build --release
```

# Future plans

Not much. If I ever feel like it, some important/quality-of-life features to add include:

- undo/redo
- 4bpp backgrounds
- select mode on tilemap
- different import/export file formats
- layout resposive to window resize

