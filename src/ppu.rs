pub struct PPU {
  cycles: u32,
  // registers
  pub ctrl: u8, // $2000
  pub mask: u8, // $2001
  pub status: u8, // $2002
}

impl PPU {
  pub fn new() -> Self {
    Self {
      cycles: 0,
      ctrl: 0,
      mask: 0,
      status: 0,
    }
  }

  pub fn tick(&mut self, cycles: u8) {
    self.cycles += cycles as u32;

    // if self.cycles > XXX {
    //   //
    // }
  }

  pub fn write_ctrl(&mut self, value: u8) {
    self.ctrl = value;
  }

  pub fn read_ctrl(&self) -> u8 {
    self.ctrl
  }

}
