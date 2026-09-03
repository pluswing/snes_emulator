use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

pub struct Cartridge {
  rom: Vec<u8>,
  sram: Vec<u8>,
}

impl Cartridge {
  pub fn new(filename: &str) -> Self {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut rom = vec![0; metadata.len() as usize];
    f.read(&mut rom).expect("buffer overflow");

    // title
    // println!("{:02X?}", &rom[0xFFC0..=0xFFD4]);
    println!("MAPPING MODE: {:02X}, CHIPSET: {:02X}, ROM: 1<<{}KB, RAM: 1<<{}KB", rom[0xFFD5], rom[0xFFD6], rom[0xFFD7], rom[0xFFD8]);
    if rom[0xFFDA] == 0x33 {
      // 後期型拡張ヘッダあり
      println!("後期型拡張ヘッダ: {:02X?}", &rom[0xFFB0..=0xFFBF]);
    }
    // MM: 0x31 = 0b0011_0001
    // CS: 0x02 = ROM+RAM+Battery
    Self {
      rom,
      sram: vec![0xFF; 0x2000], // 8KB (LoRomだけ？)
    }
  }

  fn mapping_mode(&self) -> u8 {
    self.rom[0xFFD5] & 0x0F
  }

  pub fn write(&mut self, bank: u8, addr: u16, data: u8) {
    match self.mapping_mode() {
      0x0 => {
        // LoROM/32K Banks             Mode 20 (LoROM)
        match bank {
          0x70..=0x7D => {
            match addr {
              0x0000..=0x7FFF => {
                let addr = addr / 0x2000;
                self.sram[addr as usize] = data;
              }
              0x8000..=0xFFFF => {
                panic!("should not write ROM: {:02X}:{:04X}", bank, addr)
              }
            }
          }
          0xF0..=0xFD => { // バンク$70 - $7Dのミラー
            match addr {
              0x0000..=0x7FFF => {
                let addr = addr / 0x2000;
                self.sram[addr as usize] = data;
              }
              0x8000..=0xFFFF => {
                panic!("should not write ROM: {:02X}:{:04X}", bank, addr)
              }
            }
          }
          0xFE..=0xFF => {
            match addr {
              0x0000..=0x7FFF => {
                let addr = addr / 0x2000;
                self.sram[addr as usize] = data
              }
              0x8000..=0xFFFF => {
                panic!("should not write ROM: {:02X}:{:04X}", bank, addr)
              }
            }
          }
          _ => panic!("should not write ROM: {:02X}:{:04X}", bank, addr)
        }
      }
      0x1 => {
        // HiROM/64K Banks             Mode 21 (HiROM)
      }
      // 2=LoROM/32K Banks + S-DD1     Mode 22 (mappable) "Super MMC"
      // 3=LoROM/32K Banks + SA-1      Mode 23 (mappable) "Emulates Super MMC"
      // 5=HiROM/64K Banks             Mode 25 (ExHiROM)
      // A=HiROM/64K Banks + SPC7110   Mode 25? (mappable)
      mode => panic!("invalid mapping mode {:02X}", mode)
    }
  }

  pub fn read(&self, bank: u8, addr: u16) -> u8 {
    match self.mapping_mode() {
      0x0 => {
        // LoROM/32K Banks             Mode 20 (LoROM)
        match bank {
          0x00..=0x3F => {
            match addr {
              0x8000..=0xFFFF => {
                // $00: $000000 - $007FFF
                // $01: $008000 - $00FFFF
                // $02: $010000 - $017FFF
                let addr = ((bank as u32) << 15) | ((addr as u32) & 0x7FFF);
                self.rom[addr as usize]
              }
              _ => panic!("should not reach ROM: {:02X}:{:04X}", bank, addr)
            }
          }
          0x40..=0x6F => {
            // $0000～$7FFF
            // $40: $200000 - $207FFF
            // $41: $208000 - $20FFFF
            // $42: $210000 - $217FFF
            // $8000～$FFFF
            // $40: $200000 - $207FFF
            // $41: $208000 - $20FFFF
            // $42: $210000 - $217FFF
            let addr = ((bank as u32) << 15) | ((addr as u32) & 0x7FFF);
            self.rom[addr as usize]
          }
          0x70..=0x7D => {
            match addr {
              0x0000..=0x7FFF => {
                let addr = addr / 0x2000;
                self.sram[addr as usize]
              }
              0x8000..=0xFFFF => {
                // $70: $380000 - $387FFF
                // $71: $388000 - $38FFFF
                // $72: $390000 - $397FFF
                let addr = ((bank as u32) << 15) | ((addr as u32) & 0x7FFF);
                self.rom[addr as usize]
              }
            }
          }
          0x80..=0xBF => { // バンク$00 - $3Fのミラー
            match addr {
              0x8000..=0xFFFF => {
                let addr = (((bank - 0x80) as u32) << 15) | ((addr as u32) & 0x7FFF);
                self.rom[addr as usize]
              }
              _ => panic!("should not reach ROM: {:02X}:{:04X}", bank, addr)
            }
          }
          0xC0..=0xEF => { // バンク$40 - $6Fのミラー
            let addr = (((bank - 0x80) as u32) << 15) | ((addr as u32) & 0x7FFF);
            self.rom[addr as usize]
          },
          0xF0..=0xFD => { // バンク$70 - $7Dのミラー
            match addr {
              0x0000..=0x7FFF => {
                let addr = addr / 0x2000;
                self.sram[addr as usize]
              }
              0x8000..=0xFFFF => {
                let addr = (((bank - 0x80) as u32) << 15) | ((addr as u32) & 0x7FFF);
                self.rom[addr as usize]
              }
            }
          }
          0xFE..=0xFF => {
            match addr {
              0x0000..=0x7FFF => {
                let addr = addr / 0x2000;
                self.sram[addr as usize]
              }
              0x8000..=0xFFFF => {
                // $7E: $3F0000 - $3F7FFF
                // $7F: $3F8000 - $3FFFFF
                let addr = (((bank - 0x80) as u32) << 15) | ((addr as u32) & 0x7FFF);
                self.rom[addr as usize]
              }
            }
          }
          _ => panic!("should not reach ROM: {:02X}:{:04X}", bank, addr)
        }
      }
      0x1 => {
        // HiROM/64K Banks             Mode 21 (HiROM)
        0
      }
      // 2=LoROM/32K Banks + S-DD1     Mode 22 (mappable) "Super MMC"
      // 3=LoROM/32K Banks + SA-1      Mode 23 (mappable) "Emulates Super MMC"
      // 5=HiROM/64K Banks             Mode 25 (ExHiROM)
      // A=HiROM/64K Banks + SPC7110   Mode 25? (mappable)
      mode => panic!("invalid mapping mode {:02X}", mode)
    }
  }
}
