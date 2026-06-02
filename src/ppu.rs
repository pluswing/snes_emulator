pub struct PPU {
  // registers
  pub ctrl: u8, // $2000
  pub mask: u8, // $2001
  pub status: u8, // $2002
}

impl PPU {
  pub fn new() -> Self {
    Self {
      ctrl: 0,
      mask: 0,
      status: 0,
    }
  }

  pub fn write_ctrl(&mut self, value: u8) {
    self.ctrl = value;
  }

  pub fn read_ctrl(&self) -> u8 {
    self.ctrl
  }

}
