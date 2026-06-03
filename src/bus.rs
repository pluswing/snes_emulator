use crate::ppu::PPU;

pub struct Bus {
  cpu_vram: [u8; 2048],
  // FIXME とりあえず
  pub memory: Vec<u8>, // size=0xFFFFFF
  ppu: PPU,
}

impl Bus {
  pub fn new() -> Self {
    Self {
      cpu_vram: [0; 2048],
      memory: vec![0; 0x100_0000],
      ppu: PPU::new(),
    }
  }
}

pub trait Mem {
  fn mem_read(&mut self, addr: u32) -> u8;
  fn mem_write(&mut self, addr: u32, data: u8);
}

impl Mem for Bus {
  fn mem_read(&mut self, addr: u32) -> u8 {
    match addr {
      0x2000 => self.ppu.read_ctrl(),
      _ => self.memory[addr as usize]
        // panic!("not implemented mem_read(0x{:06X})", addr)
    }
  }

  fn mem_write(&mut self, addr: u32, data: u8) {
    match addr {
      0x2000 => self.ppu.write_ctrl(data),
      _ => self.memory[addr as usize] = data
        // panic!("not implemented mem_write(0x{:06X}, ...)", addr)
    }
  }
}
