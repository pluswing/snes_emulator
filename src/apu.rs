use core::panic;
use std::ops::Add;


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
pub enum AddressingMode {
  Immediate,
  RegisterA,
  RegisterX,
  RegisterY,
  RegisterYA,
  Absolute_Indexed_by_X, // Absolute Indexed by X
  DirectPage,
}

const BOOTROM: [u8; 64] = [
  0xCD, 0xEF, 0xBD, 0xE8, 0x00, 0xC6, 0x1D, 0xD0, 0xFC, 0x8F, 0xAA, 0xF4, 0x8F, 0xBB, 0xF5, 0x78,
  0xCC, 0xF4, 0xD0, 0xFB, 0x2F, 0x19, 0xEB, 0xF4, 0xD0, 0xFC, 0x7E, 0xF4, 0xD0, 0x0B, 0xE4, 0xF5,
  0xCB, 0xF4, 0xD7, 0x00, 0xFC, 0xD0, 0xF3, 0xAB, 0x01, 0x10, 0xEF, 0x7E, 0xF4, 0x10, 0xEB, 0xBA,
  0xF6, 0xDA, 0x00, 0xBA, 0xF4, 0xC4, 0xF4, 0xDD, 0x5D, 0xD0, 0xDB, 0x1F, 0x00, 0x00, 0xC0, 0xFF
];

pub struct APU {
  pub program_counter: u16,
  ya: u16,
  x: u8,
  pub stack_pointer: u8,
  pub program_status: u8,
  memory: Vec<u8>,

  // 2140h RW - APUI00  - Main CPU to Sound CPU Communication Port 0
  // 2141h RW - APUI01  - Main CPU to Sound CPU Communication Port 1
  // 2142h RW - APUI02  - Main CPU to Sound CPU Communication Port 2
  // 2143h RW - APUI03  - Main CPU to Sound CPU Communication Port 3
  input: [u8; 4],
  output: [u8; 4],
}

impl APU {
  pub fn new() -> Self {

    let mut memory = vec![0; 0x10000];
    for (i, data) in BOOTROM.iter().enumerate() {
      memory[0xFFC0 + i] = *data;
    }

    Self {
      program_counter: 0xFFC0,
      ya: 0,
      x: 0,
      stack_pointer: 0,
      program_status: 0,
      memory,
      input: [0x00; 4],
      output: [0x00; 4],
    }
  }

  pub fn tick(&mut self, cycles: u32) {
    // FIXME cyclesを考慮
    self.run();
  }

  pub fn run(&mut self) {
    let op = self.memory[self.program_counter as usize];
    self.inc_program_counter();
    println!("APU run OP: {:04X}", op);
    match op {
      0x00 => self.nop(),
      0xCD => self.mov_x(&AddressingMode::Immediate),
      0xBD => self.mov_sp(&AddressingMode::RegisterX),
      0xE8 => self.mov_a(&AddressingMode::Immediate),
      0xC6 => self.mov_ix(&AddressingMode::RegisterA),
      0x1D => self.dec(&AddressingMode::RegisterX),
      0xD0 => self.bne(),
      0x8F => self.mov_m(&AddressingMode::Immediate),
      0x78 => self.cmp_m(&AddressingMode::Immediate),
      0x2F => self.bra(),
      0xEB => self.mov_y(&AddressingMode::DirectPage),
      0x7E => self.cmp_y(&AddressingMode::DirectPage),
      0xE4 => self.mov_a(&AddressingMode::DirectPage),
      0xCB => self.mov_m(&AddressingMode::RegisterY),
      0xD7 => self.mov_imy(&AddressingMode::RegisterA),
      0xFC => self.inc(&AddressingMode::RegisterY),
      0xAB => self.inc(&AddressingMode::DirectPage),
      0x10 => self.bpl(),
      0xBA => self.movw(&AddressingMode::RegisterYA, &AddressingMode::DirectPage),
      0xDA => self.movw(&AddressingMode::DirectPage, &AddressingMode::RegisterYA),
      0xC4 => self.mov_m(&AddressingMode::RegisterA),
      0xDD => self.mov(&AddressingMode::RegisterA, &AddressingMode::RegisterY),
      0x5D => self.mov(&AddressingMode::RegisterX, &AddressingMode::RegisterA),
      0x1F => self.jmp(&AddressingMode::Absolute_Indexed_by_X),
      _ => panic!("not implement op: {:02X}", op)
    }
  }

  fn nop(&self) {
    // なにもしない
  }

  /*
  -$CD $EF ->  MOV X, #$EF
  -$BD -> MOV SP, X
  -$E8 $00 -> MOV A, #$00
  -$C6 -> MOV (X),A
  -$1D -> DEC X
  -$D0 $FC -> BNE -
  -$8F $AA $F4 -> MOV $F4,#$AA
  -$8F $BB $F5 -> MOV $F5,#$BB
  -$78 $CC $F4 -> CMP $F4,#$CC
  -$D0 $FB -> BNE -
  -$2F $19 -> BRA Start
  -$EB $F4 -> MOV Y,$F4
  -$D0 $FC -> BNE Trans
  -$7E $F4 -> CMP Y,$F4
  -$D0 $0B -> BNE +
  -$E4 $F5 -> MOV A,$F5
  -$CB $F4 -> MOV $F4,Y
  -$D7 $00 -> MOV [$00]+Y,A
  -$FC -> INC Y
  -$D0 $F3 -> BNE -
  -$AB $01 -> INC $01
  -$10 $EF -> BPL -
  -$7E $F4 -> CMP Y,$F4
  -$10 $EB -> BPL -
  -$BA $F6 -> MOVW YA,$F6
  -$DA $00 -> MOVW $00,YA
  -$BA $F4 -> MOVW YA,$F4
  -$C4 $F4 -> MOV $F4,A
  -$DD -> MOV A,Y
  -$5D -> MOV X,A
  -$D0 $DB -> BNE Trans
  -$1F $00 $00 -> JMP [$0000+X]
  $C0 $FF -> .DW $FFC0
  */

  // MOV X, #$EF
  fn mov_x(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::Immediate => {
        let x = self.mem_read(self.program_counter);
        self.set_register_x(x);
        self.inc_program_counter();
      }
      _ => panic!("not implemented mov_x")
    }
    self.update_negative_and_zero_flags(self.get_register_x());
  }

  // MOV Y,$F4
  fn mov_y(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::DirectPage => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let v = self.mem_read(self.direct_page_addr(addr));
        self.set_register_y(v);
      }
      _ => panic!("not implemented mov_y")
    }
    self.update_negative_and_zero_flags(self.get_register_y());
  }

  // MOV SP, X
  fn mov_sp(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::RegisterX => {
        self.stack_pointer = self.get_register_x();
      }
      _ => panic!("not implemented mov_sp")
    }
    // TODO update_negetive_and_zero_flags?
  }

  // MOV A, #$00
  fn mov_a(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::Immediate => {
        let v = self.mem_read(self.program_counter);
        self.inc_program_counter();
        self.set_register_a(v);
      }
      AddressingMode::DirectPage => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let v = self.mem_read(self.direct_page_addr(addr));
        self.set_register_a(v);
      }
      _ => panic!("not implemented mov_a")
    }
    self.update_negative_and_zero_flags(self.get_register_a());
  }

  // MOV (X),A
  fn mov_ix(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::RegisterA => {
        let addr = self.direct_page_addr(self.get_register_x());
        self.mem_write(addr, self.get_register_a());
      }
      _ => panic!("not implemented mov_ix")
    }
  }
  // MOV [$00]+Y,A
  fn mov_imy(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::RegisterA => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let addr = self.direct_page_addr(addr);
        let addr = self.mem_read_u16(addr) as u16;
        let addr = addr.wrapping_add(self.get_register_y() as u16);
        self.mem_write(addr, self.get_register_a());
      }
      _ => panic!("not implemented mov_imy")
    }
  }

  // MOV $F4,#$AA
  // MOV $F4,Y
  fn mov_m(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::Immediate => {
        let data = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let dest = self.mem_read(self.program_counter);
        let dest = self.direct_page_addr(dest);
        self.inc_program_counter();
        self.mem_write(dest as u16, data);
      }
      AddressingMode::RegisterY => {
        let dest = self.mem_read(self.program_counter);
        let dest = self.direct_page_addr(dest);
        self.inc_program_counter();
        self.mem_write(dest as u16, self.get_register_y());
      }
      AddressingMode::RegisterA => {
        let dest = self.mem_read(self.program_counter);
        let dest = self.direct_page_addr(dest);
        self.inc_program_counter();
        self.mem_write(dest as u16, self.get_register_a());
      }
      _ => panic!("not implemented mov_m")
    }
  }

  // MOV A,Y
  // MOV X,A
  fn mov(&mut self, dest: &AddressingMode, src: &AddressingMode) {
    let data = match src {
      AddressingMode::RegisterY => {
        self.get_register_y()
      }
      AddressingMode::RegisterA => {
        self.get_register_a()
      }
      _ => panic!("not implemented mov")
    };
    match dest {
      AddressingMode::RegisterA => {
        self.set_register_a(data);
        self.update_negative_and_zero_flags(data);
      }
      AddressingMode::RegisterX => {
        self.set_register_x(data);
        self.update_negative_and_zero_flags(data);
      }
      _ => panic!("not implemented mov")
    }
  }

  // MOVW YA,$F6
  fn movw(&mut self, dest: &AddressingMode, src: &AddressingMode) {
    let data = match src {
      AddressingMode::DirectPage => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let addr = self.direct_page_addr(addr);
        let data = self.mem_read_u16(addr);
        data
      }
      AddressingMode::RegisterYA => {
        self.ya
      }
      _ => panic!("not implemented movw")
    };
    match dest {
      AddressingMode::RegisterYA => {
        self.ya = data;
        self.update_negative_and_zero_flags_u16(data);
      }
      AddressingMode::DirectPage => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let addr = self.direct_page_addr(addr);
        self.mem_write_u16(addr, data);
      }
      _ => panic!("not implemented movw")
    }
  }

  // DEC X
  fn dec(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::RegisterX => {
        let x = self.get_register_x().wrapping_sub(1);
        self.set_register_x(x);
        self.update_negative_and_zero_flags(x);
      }
      _ => panic!("not implemented dec")
    }
  }

  // INC Y
  // INC $01
  fn inc(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::RegisterY => {
        let y = self.get_register_y().wrapping_add(1);
        self.set_register_y(y);
        self.update_negative_and_zero_flags(y);
      }
      AddressingMode::DirectPage => {
        let addr = self.mem_read(self.program_counter);
        let addr = self.direct_page_addr(addr);
        self.inc_program_counter();
        let data = self.mem_read(addr);
        let data = data.wrapping_add(1);
        self.mem_write(addr, data);
        self.update_negative_and_zero_flags(data);
      }
      _ => panic!("not implemented inc")
    }
  }

  // BNE $FC
  fn bne(&mut self) {
    let v = self.mem_read(self.program_counter) as i8;
    let v = v as i16;
    let v = v as u16;
    self.inc_program_counter();

    if (self.program_status & FLAG_ZERO) != 0 {
      return
    }
    self.program_counter = self.program_counter.wrapping_add(v)
  }

  // BRA Start
  fn bra(&mut self) {
    let v = self.mem_read(self.program_counter) as i8;
    let v = v as i16;
    let v = v as u16;
    self.inc_program_counter();
    self.program_counter = self.program_counter.wrapping_add(v)
  }

  // BPL -
  fn bpl(&mut self) {
    let v = self.mem_read(self.program_counter) as i8;
    let v = v as i16;
    let v = v as u16;
    self.inc_program_counter();

    if (self.program_status & FLAG_NEGATIVE) != 0 {
      return
    }
    self.program_counter = self.program_counter.wrapping_add(v)
  }

  // CMP $F4,#$CC
  fn cmp_m(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::Immediate => {
        let right = self.mem_read(self.program_counter);
        self.inc_program_counter();

        let dest = self.mem_read(self.program_counter);
        let left = self.mem_read(self.direct_page_addr(dest));
        self.inc_program_counter();

        // N、Z、Cフラグが変更されます。
        let (v, c) = left.overflowing_sub(right);
        self.program_status = if !c {
          self.program_status | FLAG_CARRY
        } else {
          self.program_status & !FLAG_CARRY
        };
        self.update_negative_and_zero_flags(v);
      }
      _ => panic!("not implemented cmp_m")
    }
  }

  // CMP Y,$F4
  fn cmp_y(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::DirectPage => {
        let dest = self.mem_read(self.program_counter);
        let right = self.mem_read(self.direct_page_addr(dest));
        self.inc_program_counter();

        let left = self.get_register_y();
        let (v, c) = left.overflowing_sub(right);
        self.program_status = if !c {
          self.program_status | FLAG_CARRY
        } else {
          self.program_status & !FLAG_CARRY
        };
        self.update_negative_and_zero_flags(v);
      }
      _ => panic!("not implemented cmp_y")
    }
  }

  // JMP [$0000+X]
  fn jmp(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::Absolute_Indexed_by_X => {
        let addr = self.mem_read_u16_no_wrapped(self.program_counter);
        let x = self.get_register_x();
        let addr = addr.wrapping_add(x as u16);
        let addr = self.mem_read_u16_no_wrapped(addr);
        self.program_counter = addr;
      }
      _ => panic!("not implemented jmp")
    }
  }

  fn inc_program_counter(&mut self) {
    self.program_counter = self.program_counter.wrapping_add(1);
  }

  fn direct_page_addr(&self, base: u8) -> u16 {
    if (self.program_status & FLAG_DIRECT_PAGE) != 0 {
      0x0100 | base as u16
    } else {
      base as u16
    }
  }

  fn update_negative_and_zero_flags(&mut self, result: u8) {
    let test_bit = 0x80;
    self.program_status = if result == 0 {
        self.program_status | FLAG_ZERO
    } else {
        self.program_status & !FLAG_ZERO
    };
    self.program_status = if (result & test_bit) != 0 {
        self.program_status | FLAG_NEGATIVE
    } else {
        self.program_status & !FLAG_NEGATIVE
    }
  }

  fn update_negative_and_zero_flags_u16(&mut self, result: u16) {
    let test_bit = 0x8000;
    self.program_status = if result == 0 {
        self.program_status | FLAG_ZERO
    } else {
        self.program_status & !FLAG_ZERO
    };
    self.program_status = if (result & test_bit) != 0 {
        self.program_status | FLAG_NEGATIVE
    } else {
        self.program_status & !FLAG_NEGATIVE
    }
  }

  pub fn get_register_a(&self) -> u8 {
    (self.ya & 0x00FF) as u8
  }

  pub fn set_register_a(&mut self, value: u8) {
    self.ya = (self.ya & 0xFF00) | (value as u16)
  }

  pub fn get_register_y(&self) -> u8 {
    (self.ya >> 8) as u8
  }

  pub fn set_register_y(&mut self, value: u8) {
    self.ya = (self.ya & 0x00FF) | ((value as u16) << 8)
  }

  pub fn get_register_x(&self) -> u8 {
    self.x
  }

  pub fn set_register_x(&mut self, value: u8) {
    self.x = value
  }

  pub fn mem_read(&mut self, addr: u16) -> u8 {
    match addr {
      0x00F4..=0x00F7 => {
        let port = (addr - 0x00F4) % 4;
        self.input[port as usize]
      }
      _ => self.memory[addr as usize]
    }
  }

  pub fn mem_write(&mut self, addr: u16, data: u8) {
    match addr {
      0x00F4..=0x00F7 => {
        let port = (addr - 0x00F4) % 4;
        self.output[port as usize] = data
      }
      _ => self.memory[addr as usize] = data
    }
  }

  pub fn mem_read_u16(&mut self, addr: u16) -> u16 {
    let lo = self.mem_read(addr) as u16;
    let addr = (addr & 0xFF00) | ((addr + 1) & 0x00FF);
    let hi = self.mem_read(addr) as u16;
    hi << 8 | lo
  }

  pub fn mem_read_u16_no_wrapped(&mut self, addr: u16) -> u16 {
    let lo = self.mem_read(addr) as u16;
    let addr = addr.wrapping_add(1);
    let hi = self.mem_read(addr) as u16;
    hi << 8 | lo
  }

  pub fn mem_write_u16(&mut self, addr: u16, data: u16) {
    self.mem_write(addr, (data & 0x00FF) as u8);
    let addr = (addr & 0xFF00) | ((addr + 1) & 0x00FF);
    self.mem_write(addr, (data >> 8) as u8);
  }

  pub fn write(&mut self, addr: u16, data: u8) {
    println!("APU write({:04X}, {:02X})", addr, data);
    match addr {
      0x2140..=0x217F => {
        let port = (addr - 0x2140) % 4;
        self.input[port as usize] = data;
      }
      _ => panic!("should not reach. APU::write"),
    }
  }

  pub fn read(&mut self, addr: u16) -> u8 {
    println!("APU read({:04X})", addr);
    match addr {
      0x2140..=0x217F => {
        // 2140h RW - APUI00  - Main CPU to Sound CPU Communication Port 0        (00h/00h)
        // 2141h RW - APUI01  - Main CPU to Sound CPU Communication Port 1        (00h/00h)
        // 2142h RW - APUI02  - Main CPU to Sound CPU Communication Port 2        (00h/00h)
        // 2143h RW - APUI03  - Main CPU to Sound CPU Communication Port 3        (00h/00h)
        // 2144h..217Fh    - APU Ports 2140-2143h mirrored to 2144h..217Fh
        let port = (addr - 0x2140) % 4;
        println!("  {:02X} => {:02X}", port, self.output[port as usize]);
        self.output[port as usize]
      }
      _ => panic!("should not reach. APU::read"),
    }
  }
}
