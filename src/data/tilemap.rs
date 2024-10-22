use std::fs::File;
use std::io::Write;

use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Tile {
    pub tile_idx: B10,
    pub palette: B3,
    pub priority: bool,
    pub x_flip: bool,
    pub y_flip: bool,
}

pub struct Tilemap(pub [Tile; 1024]);

impl Default for Tilemap {
    fn default() -> Self {
        Self([Tile::default(); 1024])
    }
}

impl Tilemap {
    pub fn from_file(path: &std::path::PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read(&path)?;
        let len = content.len();
        // check alignment
        if len % 2 != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("file size is {} but should be a multiple of 2", len),
            ));
        }
        // check file size
        if len > 32 * 32 * 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "file must not exceed 2048 bytes",
            ));
        }

        Ok(Tilemap({
            let mut v = [Tile::default(); 1024];
            for i in (0..len).step_by(2) {
                v[i / 2] = Tile::from_bytes([content[i], content[i + 1]]);
            }
            v
        }))
    }

    pub fn write_to_file(&self, mut f: &File) -> std::io::Result<()> {
        for c in self.0 {
            f.write_all(&c.into_bytes())?;
        }
        Ok(())
    }
}
