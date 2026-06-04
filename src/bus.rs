use crate::ppu::PPU;

pub struct Bus {
  wram: Vec<u8>,
  ppu: PPU,

  // FIXME とりあえず
  pub memory: Vec<u8>, // size=0xFFFFFF
}

impl Bus {
  pub fn new() -> Self {
    Self {
      wram: vec![0; 0x1_0000 * 2],
      ppu: PPU::new(),

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
    match addr {
      0x2000 => self.ppu.read_ctrl(),
      0x7E0000..=0x7FFFFF => {
        self.wram[addr as usize - 0x7E0000]
      }
      _ => self.memory[addr as usize]
        // panic!("not implemented mem_read(0x{:06X})", addr)
    }
  }

  fn mem_write(&mut self, addr: u32, data: u8) {
    match addr {
      0x2000 => self.ppu.write_ctrl(data),
      0x7E0000..=0x7FFFFF => {
        self.wram[addr as usize - 0x7E0000] = data
      }
      _ => self.memory[addr as usize] = data
        // panic!("not implemented mem_write(0x{:06X}, ...)", addr)
    }
  }
}
