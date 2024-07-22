# Waffle

Tilemap editor for SNES backgrounds

![waffle](https://github.com/user-attachments/assets/65aed4da-0b0c-478d-9f89-3c7582f87771)

# Build

~~works on my machine~~ 

## Linux

```sh
#gtk version 4.6, but should(?) work for versions < 4.10
apt install libgtk-4-dev build-essential
#make sure rustup is installed, then
cargo build --release
```

# File format

File format for palette, tile set, and tile map is the same as the format used in CGRAM/VRAM, so it is possible to include the binary files directly in assembly code without further processing.

# Future plans

Not much. If I ever feel like it, some important/quality-of-life features to add include:

- undo/redo
- 4bpp backgrounds
- select mode on tilemap
- different import/export file formats
- options on how to display transparent colors

