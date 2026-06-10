use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

pub struct Cartridge {
  rom: Vec<u8>,
}

impl Cartridge {
  pub fn new(filename: &str) -> Self {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut rom = vec![0; metadata.len() as usize];
    f.read(&mut rom).expect("buffer overflow");

    println!("{:02X?}", &rom[0xFFC0..=0xFFD4]);
    Self {
      rom
    }
  }

  fn read(&self, addr: u32) -> u8 {
    self.rom[addr as usize]
  }
}
