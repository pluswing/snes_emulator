
const FLAG_NEGATIVE: u8 = 1 << 7;
const FLAG_OVERFLOW: u8 = 1 << 6;
const FLAG_DIRECT_PAGE: u8 = 1 << 5;
// 4 なし
const FLAG_HALF_CARRY: u8 = 1 << 3;
// 2 なし
const FLAG_ZERO: u8 = 1 << 1;
const FLAG_CARRY: u8 = 1 << 0;

pub struct APU {
  // FIXME あとでけす
  status: u8,
  counter: u32,

  program_counter: u16,
  ya: u16,
  x: u8,
  stack_pointer: u8,
  program_status: u8,
  memory: Vec<u8>,
}

impl APU {
  pub fn new() -> Self {
    Self {
      status: 0xAA,
      counter: 0,

      program_counter: 0xFFC0,
      ya: 0,
      x: 0,
      stack_pointer: 0,
      program_status: 0,
      memory: vec![0; 0x10000],
    }
  }

  fn tick(&mut self, cycles: u32) {
    // FIXME
  }

  fn run(&mut self) {
    let op = self.memory[self.program_counter as usize];
    match op {
      0x00 => self.nop(),
      _ => panic!("not implement op: {:02X}", op)
    }
  }

  fn nop(&self) {
    // なにもしない
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
        if self.counter == 200 {
          self.status = 0xAA;
          self.counter = 201;
        }
        if self.counter == 1310 {
          self.status = 0xCC;
          self.counter = 1311;
        }
        if self.counter == 30 {
          self.counter = 101;
          self.status = 0x00;
        }
        if self.counter < 65535 {
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
