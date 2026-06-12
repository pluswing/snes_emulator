use crate::{cartridge::{self, Cartridge}, ppu::PPU};

pub struct Bus {
  wram: Vec<u8>,
  ppu: PPU,
  cartridge: Cartridge,

  // FIXME とりあえず
  pub memory: Vec<u8>, // size=0xFFFFFF
}

impl Bus {
  pub fn new(ppu: PPU, cartridge: Cartridge) -> Self {
    Self {
      wram: vec![0; 0x1_0000 * 2],
      ppu,
      cartridge,
      memory: vec![0; 0x100_0000],
    }
  }

  pub fn tick(&mut self, cycles: u8) {
    self.ppu.tick(cycles);
  }
}

pub trait Mem {
  fn mem_read(&mut self, addr: u32) -> u8;
  fn mem_write(&mut self, addr: u32, data: u8);
}

// [0x000000 ~ 0xFFFFFF]
// 0x0000~0xFFFF => 64KB * 2

impl Mem for Bus {
  fn mem_read(&mut self, addr: u32) -> u8 {
    let bank = ((addr & 0xFF_0000) >> 16) as u8;
    let addr = (addr & 0x00_FFFF) as u16;
    match bank {
      0x00..=0x3F => {
        match addr {
          0x0000..0x1FFF => self.wram[addr as usize],
          0x8000..=0xFFFF => self.cartridge.read(bank, addr),
          _ => panic!("not implemented mem_read({:02X}:{:04X})", bank, addr)
        }
      }
      0x40..=0x7D => {
        self.cartridge.read(bank, addr)
      }
      0x7E..=0x7F => {
        self.wram[addr as usize]
      }
      0x80..=0xBF => {
        match addr {
          0x0000..0x1FFF => self.wram[addr as usize],
          0x8000..=0xFFFF => self.cartridge.read(bank, addr),
          _ => panic!("not implemented mem_read({:02X}:{:04X})", bank, addr)
        }
      }
      0xC0..=0xFF => {
        self.cartridge.read(bank, addr)
      }
      _ => panic!("not implemented mem_read({:02X}:{:04X})", bank, addr)
    }
  }

  fn mem_write(&mut self, addr: u32, data: u8) {
    match addr {
      0x7E0000..=0x7FFFFF => {
        self.wram[addr as usize - 0x7E0000] = data
      }
      _ => self.memory[addr as usize] = data
        // panic!("not implemented mem_write(0x{:06X}, ...)", addr)
    }
  }
}
