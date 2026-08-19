pub struct APU {
}

impl APU {
  pub fn new() -> Self {
    Self {
    }
  }

  pub fn write(&mut self, addr: u16, data: u8) {
  }
  pub fn read(&mut self, addr: u16) -> u8 {
  }
  // 2140h RW - APUI00  - Main CPU to Sound CPU Communication Port 0        (00h/00h)
  // 2141h RW - APUI01  - Main CPU to Sound CPU Communication Port 1        (00h/00h)
  // 2142h RW - APUI02  - Main CPU to Sound CPU Communication Port 2        (00h/00h)
  // 2143h RW - APUI03  - Main CPU to Sound CPU Communication Port 3        (00h/00h)
  // 2144h..217Fh    - APU Ports 2140-2143h mirrored to 2144h..217Fh
}
