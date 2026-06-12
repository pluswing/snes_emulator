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

  fn mapping_mode(&self) -> u8 {
    (self.rom[0xFFD5] & 0x08)
  }

  pub fn read(&self, bank: u8, addr: u16) -> u8 {
    match self.mapping_mode() {
      0x0 => {
        // LoROM/32K Banks             Mode 20 (LoROM)
        match bank {
          0x00..=0x2F => {
            match addr {
              0x8000..=0xFFFF => self.rom[addr as usize - 0x8000]
              _ => panic!("should not reach")
            }
          }
          0x30..=0x3F => {
            match addr {
              0x8000..=0xFFFF => self.rom[addr as usize - 0x8000]
              _ => panic!("should not reach")
            }
          }
          0x40..=0x5F => {
            match addr {
              0x8000..=0xFFFF => self.rom[addr as usize - 0x8000]
              _ => panic!("should not reach")
            }
          }
          0x70..=0x77	=> {
            match addr {
              0x0000..=0x7FFF => {
                0 // FIXME Mode 20 SRAM (256Kバイト)
              }
            }
          }
          0x80..=0xDF => {
            // FIXME: mirror
          }
        }
      }
      0x1 => {
        // HiROM/64K Banks             Mode 21 (HiROM)
        0
      }
      mode => panic!("invalid mapping mode {:02X}", mode)
    }
  }
}
