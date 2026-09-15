
const FLAG_NEGATIVE: u8 = 1 << 7;
const FLAG_OVERFLOW: u8 = 1 << 6;
const FLAG_DIRECT_PAGE: u8 = 1 << 5;
// 4 なし
const FLAG_HALF_CARRY: u8 = 1 << 3;
// 2 なし
const FLAG_ZERO: u8 = 1 << 1;
const FLAG_CARRY: u8 = 1 << 0;

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
enum AddressingMode {
  Immediate,
  RegisterA,
  RegisterX,
  IndirectX,
}

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
    self.program_counter += 1;
    match op {
      0x00 => self.nop(),
      0xCD => self.mov_x(&AddressingMode::Immediate),
      0xBD => self.mov_sp(&AddressingMode::RegisterX),
      0xE8 => self.mov_a(&AddressingMode::Immediate),
      0xC6 => self.move_ix(&AddressingMode::RegisterA),
      _ => panic!("not implement op: {:02X}", op)
    }
  }

  fn nop(&self) {
    // なにもしない
  }

  /*
  $CD $EF ->  MOV X, #$EF
  $BD -> MOV SP, X
  $E8 $00 -> MOV A, #$00
  $C6 -> MOV (X),A
  $1D $D0 $FC $8F $AA $F4 $8F $BB $F5 $78
  $CC $F4 $D0 $FB $2F $19 $EB $F4 $D0 $FC $7E $F4 $D0 $0B $E4 $F5
  $CB $F4 $D7 $00 $FC $D0 $F3 $AB $01 $10 $EF $7E $F4 $10 $EB $BA
  $F6 $DA $00 $BA $F4 $C4 $F4 $DD $5D $D0 $DB $1F $00 $00 $C0 $FF
  */

  // MOV X, #$EF
  fn mov_x(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::Immediate => {
        self.x = self.mem_read(self.program_counter);
        self.program_counter += 1;
      }
      _ => panic!("not implemented mov_x")
    }
  }

  // MOV SP, X
  fn mov_sp(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::RegisterX => {
        self.stack_pointer = self.get_register_x();
      }
      _ => panic!("not implemented mov_x")
    }
  }

  // MOV A, #$00
  fn mov_a(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::Immediate => {
        let v = self.mem_read(self.program_counter);
        self.program_counter += 1;
        self.set_register_a(v);
      }
      _ => panic!("not implemented mov_a")
    }
  }

  // MOV (X),A
  fn move_ix(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::RegisterA => {
        self.mem_write(self.get_register_x() as u16, self.get_register_a());
      }
      _ => panic!("not implemented mov_a")
    }
  }

  fn get_register_a(&self) -> u8 {
    (self.ya & 0x00FF) as u8
  }

  fn set_register_a(&mut self, value: u8) {
    self.ya = (self.ya & 0xFF00) | (value as u16)
  }

  fn get_register_y(&self) -> u8 {
    (self.ya >> 8) as u8
  }

  fn set_register_y(&mut self, value: u8) {
    self.ya = (self.ya & 0x00FF) | ((value as u16) << 8)
  }

  fn get_register_x(&self) -> u8 {
    self.x
  }

  fn set_register_x(&mut self, value: u8) {
    self.x = value
  }

  fn mem_read(&mut self, addr: u16) -> u8 {
    self.memory[addr as usize]
  }

  fn mem_write(&mut self, addr: u16, data: u8) {
    self.memory[addr as usize] = data;
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
