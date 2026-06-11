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

    // title
    // println!("{:02X?}", &rom[0xFFC0..=0xFFD4]);
    println!("MAPPING MODE: {:02X}, CHIPSET: {:02X}, ROM: {}KB, RAM: {}KB", rom[0xFFD5], rom[0xFFD6], 1 << rom[0xFFD7], 1 << rom[0xFFD8]);
    if rom[0xFFDA] == 0x33 {
      // 後期型拡張ヘッダあり
      println!("後期型拡張ヘッダ: {:02X?}", &rom[0xFFB0..=0xFFBF]);
    }
    // MM: 0x31 = 0b0011_0001
    // CS: 0x02 = ROM+RAM+Battery
    Self {
      rom
    }
  }

  fn read(&self, addr: u32) -> u8 {
    self.rom[addr as usize]
  }
}
