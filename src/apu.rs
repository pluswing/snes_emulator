pub struct APU {
  status: u8,
  counter: u8,
}

impl APU {
  pub fn new() -> Self {
    Self {
      status: 0xAA,
      counter: 0,
    }
  }

  pub fn write(&mut self, addr: u16, data: u8) {
    println!("APU write({:04X}, {:02X})", addr, data);
    match addr {
      0x2140 => {

      }
      0x2141 => {
        if self.counter >= 100 {
          self.status = data;
        }
      },
      0x2142 => {},
      0x2143 => {},
      _ => {},
    }
  }

  pub fn read(&mut self, addr: u16) -> u8 {
    println!("APU read({:04X})", addr);
    match addr {
      0x2140 => {
        if self.counter == 101 {
          self.status = 0xCC;
          self.counter = 102;
        }
        if self.counter == 100 {
          self.counter = 101;
          self.status = 0x00;
        }
        if self.counter < 100 {
          self.counter += 1;
        }
        self.status
      }
      0x2141 => 0xBB,
      0x2142 => 0x00,
      0x2143 => 0x00,
      _ => 0,
    }
  }
  // 2140h RW - APUI00  - Main CPU to Sound CPU Communication Port 0        (00h/00h)
  // 2141h RW - APUI01  - Main CPU to Sound CPU Communication Port 1        (00h/00h)
  // 2142h RW - APUI02  - Main CPU to Sound CPU Communication Port 2        (00h/00h)
  // 2143h RW - APUI03  - Main CPU to Sound CPU Communication Port 3        (00h/00h)
  // 2144h..217Fh    - APU Ports 2140-2143h mirrored to 2144h..217Fh
}
