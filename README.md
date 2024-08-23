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

# File format

File format for palette, tile set, and tile map is the same as the format used in CGRAM/VRAM, so it is possible to include the binary files directly in assembly code without further processing.

## Palette files

`Palette > Open` and `Palette > Save (as)` uses the BGR555 format, but other formats can be imported and exported.

### BGR555

An example of this format can be found [here](examples/palette.bin).

The file size is always 512 bytes. Each color takes up 2 bytes, and 256 colors are stored in order from 0 to 255.

For each color, red, gree, and blue takes up 5 bits each.

```text
color                 0                 1 ...
byte         0        1        2        3 ...
data  gggrrrrr xbbbbbgg gggrrrrr xbbbbbgg ...

bits 4-0 of byte 0 stores red 
bits 1-0 of byte 1 and bits 7-5 of byte 0 stores green
bits 6-2 of byte 1 stores blue
bit 7 of byte 1 can be anything
``` 

### RGB24

An example of this format can be found [here](examples/palette.pal).

The file size is always 768 bytes. Each color takes up 3 bytes, and 256 colors are stored in order from 0 to 255.

For each color, red, green, and blue takes up 8 bits each.

```text
color                          0          ...
byte         0        1        2        3 ...
data  rrrrrrrr gggggggg bbbbbbbb rrrrrrrr ...

byte 0 stores red, byte 1 stores green, and byte 2 stores blue
```

To convert from RGB24 to BGR555 (when importing), the top 5 bits of each color is kept and the rest are discarded. To convert from BGR555 to RGB24 (when exporting), the 5 bits are shifted left by 3, then repeated to fill up 8 bits of space (resulting in the bit pattern 43210432).

This is the format used by YY-CHR, by going to `Palette > Save palette(*.pal)...`.

# Future plans

Not much. If I ever feel like it, some important/quality-of-life features to add include:

- undo/redo
- 4bpp backgrounds
- select mode on tilemap
- different import/export file formats
- layout resposive to window resize
- options on how to display transparent colors
