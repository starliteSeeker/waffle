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

## Tileset files

Tileset can either be in 2bpp or 4bpp format. This can be chosen by opening the file through `Tileset > Open 2bpp` or `Tileset > Open 4bpp`.

### 2bpp

An example of this format can be found [here](examples/tileset.bin).

Each pixel takes up 2 bits, and each 8-by-8 tile takes up 16 bytes. Data is stored row-wise, with bit 0 of each pixel stored first, then bit 1 of each pixel. 

```text
tile                           ...                 0 ... 
row                  0         ...                 7 ... 
data 00000000 11111111 0000000 ... 00000000 11111111 ... 
     |      | |      |
     |      | |      `bit 1 of the rightmost pixel
     |      | `bit 1 of the leftmost pixel
     |      `bit 0 of the rightmost pixel
     `bit 0 of the leftmost pixel 
```

### 4bpp

Each pixel takes up 4 bits, and each tile takes up 32 bytes. Bits 0 and 1 of a tile is stored first, similar to the 2bpp format, then bits 2 and 3 are stored.

```text
tile                            ...                   
row                  0          ...                 7 
data 00000000 11111111 00000000 ... 00000000 11111111 
     |      | |      |
     |      | |      `bit 1 of the rightmost pixel
     |      | `bit 1 of the leftmost pixel
     |      `bit 0 of the rightmost pixel
     `bit 0 of the leftmost pixel 

tile                            ...                 0 ...
row                  0          ...                 7 ...
data 22222222 33333333 22222222 ... 22222222 33333333 ...
     |      | |      |
     |      | |      `bit 3 of the rightmost pixel
     |      | `bit 3 of the leftmost pixel
     |      `bit 2 of the rightmost pixel
     `bit 2 of the leftmost pixel 
```

## Tilemap files

An example of this format can be found [here](examples/tilemap.bin).

The file size is usually 2048 bytes. Each tilemap tile takes up 2 bytes, and tiles are arranged as a 32-by-32 square. For convenience, files storing less than 32x32 tiles are also accepted.

```text
tile                 0 ...
byte        0        1 ...
data tttttttt vhpccctt ...

bits 1-0 of byte 1 and bits 7-0 of byte 0 stores tile index
bits 4-2 of byte 1 stores palette index
bits 5 stores priority
bits 6 stores horizontal flip
bits 7 stores vertical flip
```

In the 4bpp format, only the first 128 colors can be used, with the palette index pointing to the 16-color palette used by the tile.
In the 2bpp format, only a 32-color subset can be used, with the palette index further narrowing it down to a 4-color palette. 

# Future plans

Not much. If I ever feel like it, some important/quality-of-life features to add include:

- select mode on tilemap
- different import/export file formats
- layout resposive to window resize
- options on how to display transparent colors
