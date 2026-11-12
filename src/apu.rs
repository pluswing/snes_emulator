use core::panic;

const FLAG_NEGATIVE: u8 = 1 << 7;
const FLAG_OVERFLOW: u8 = 1 << 6;
const FLAG_DIRECT_PAGE: u8 = 1 << 5;
// 4 なし
const FLAG_HALF_CARRY: u8 = 1 << 3;
const FLAG_INTERRRUPT: u8 = 1 << 2;
const FLAG_ZERO: u8 = 1 << 1;
const FLAG_CARRY: u8 = 1 << 0;

const SIGN_BIT: u8 = 0x80;
const SIGN_BIT_U16: u16 = 0x8000;

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Args {
  Immediate,
  RegisterA,
  RegisterX,
  RegisterY,
  RegisterYA,
  Absolute_Indexed_by_X, // Absolute Indexed by X
  Absolute_Indexed_by_Y, // Absolute Indexed by Y
  DirectPage,
  Absolute,
  StackPointer,
  RegisterX_Direct_Page_Indirect,
  RegisterX_Direct_Page_Indirect_Incriment,
  RegisterY_Direct_Page_Indirect,
  Direct_Page_Indirect_Indexed_by_Y, // Direct Page Indirect Indexed by Y
  Direct_Page_Indexed_Indirect_by_X, // Direct Page Indexed Indirect by X
  Direct_Page_Indexed_by_X, // Direct Page Indexed by X
  Direct_Page_Indexed_by_Y, // Direct Page Indexed by Y
  ProgramStatus,

  C/* C */,
  mem/* mem. */,
  mem2/* mem. */,
  CY/* (Y) */,
  absY/* [!abs+X] */,
  dpx/* dp+x */,
  upage/* upage */,
  bit/* bit */,
  n/* n */,
  F6/* $F6 */,
  Zero/* $00 */,
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

  _disable_inc_pc: bool,
  _disable_update_status: bool,

  unit_test_mode: bool,
}

impl APU {
  pub fn unit_test() -> Self {
    Self::_new(true)
  }

  pub fn new() -> Self {
    Self::_new(false)
  }

  fn _new(unit_test_mode: bool) -> Self {

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
      _disable_inc_pc: false,
      _disable_update_status: false,
      unit_test_mode,
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
/*
    match op {
      0x00 => self.nop(),
      0xCD => self.mov(&Args::RegisterX, &Args::Immediate),
      0xBD => self.mov(&Args::StackPointer, &Args::RegisterX),
      0xE8 => self.mov(&Args::RegisterA, &Args::Immediate),
      0xC6 => self.mov(&Args::RegisterX_Direct_Page_Indirect, &Args::RegisterA),
      0x1D => self.dec(&Args::RegisterX),
      0xD0 => self.bne(),
      0x8F => self.mov(&Args::DirectPage, &Args::Immediate),
      0x78 => self.cmp(&Args::DirectPage, &Args::Immediate),
      0x2F => self.bra(),
      0xEB => self.mov(&Args::RegisterY, &Args::DirectPage),
      0x7E => self.cmp(&Args::RegisterY, &Args::DirectPage),
      0xE4 => self.mov(&Args::RegisterA, &Args::DirectPage),
      0xCB => self.mov(&Args::DirectPage, &Args::RegisterY),
      0xD7 => self.mov(&Args::Direct_Page_Indirect_Indexed_by_Y, &Args::RegisterA),
      0xFC => self.inc(&Args::RegisterY),
      0xAB => self.inc(&Args::DirectPage),
      0x10 => self.bpl(),
      0xBA => self.movw(&Args::RegisterYA, &Args::DirectPage),
      0xDA => self.movw(&Args::DirectPage, &Args::RegisterYA),
      0xC4 => self.mov(&Args::DirectPage, &Args::RegisterA),
      0xDD => self.mov(&Args::RegisterA, &Args::RegisterY),
      0x5D => self.mov(&Args::RegisterX, &Args::RegisterA),
      0x1F => self.jmp(&Args::Absolute_Indexed_by_X),

      0x5F => self.jmp(&Args::Absolute),
      0xC0 => self.di(),
      0x3F => self.call(),
      0x6F => self.ret(),
      0x20 => self.clrp(),
      0xF5 => self.mov(&Args::RegisterA,  &Args::Absolute_Indexed_by_X),
      0xF0 => self.beq(),
      0x3D => self.inc(&Args::RegisterX),
      0xAF => self.mov(&Args::RegisterX_Direct_Page_Indirect_Incriment, &Args::RegisterA),
      0xC8 => self.cmp(&Args::RegisterX, &Args::Immediate),
      0x90 => self.bcc(),
      0x40 => self.setp(),
      0x8D => self.mov(&Args::RegisterY, &Args::Immediate),
      0x7D => self.mov(&Args::RegisterA, &Args::RegisterX),
      0xC7 => self.mov(&Args::Direct_Page_Indexed_Indirect_by_X, &Args::RegisterA),
      0x3A => self.incw(),
      0x5A => self.cmpw(&Args::RegisterYA, &Args::DirectPage),
      0xD5 => self.mov(&Args::Absolute_Indexed_by_X, &Args::RegisterA),
      0xC5 => self.mov(&Args::Absolute, &Args::RegisterA),
      0xBC => self.inc(&Args::RegisterA),
      0x60 => self.clrc(),
      0x88 => self.adc(&Args::RegisterA, &Args::Immediate),
      0x68 => self.cmp(&Args::RegisterA, &Args::Immediate),
      0xE5 => self.mov(&Args::RegisterA, &Args::Absolute),
      0xEC => self.mov(&Args::RegisterY, &Args::Absolute),
      0x9A => self.subw(&Args::RegisterYA, &Args::DirectPage),
      0x64 => self.cmp(&Args::RegisterA, &Args::DirectPage),
      0x65 => self.cmp(&Args::RegisterA, &Args::Absolute),
      0x44 => self.eor(&Args::RegisterA, &Args::DirectPage),
      0x28 => self.and(&Args::RegisterA, &Args::Immediate),
      0x30 => self.bmi(),
      0xB0 => self.bcs(),
      0x2D => self.push(&Args::RegisterA),
      0x1C => self.asl(&Args::RegisterA),
      0xAE => self.pop(&Args::RegisterA),
      0xC2 => self.set1(op),
      0x48 => self.eor(&Args::RegisterA, &Args::Immediate),
      0xB2 => self.clr1(op),
      0x80 => self.setc(),
      0xA8 => self.sbc(&Args::RegisterA, &Args::Immediate),
      0x6D => self.push(&Args::RegisterY),
      0xCF => self.mul(),
      0xFD => self.mov(&Args::RegisterY, &Args::RegisterA),
      0x9D => self.mov(&Args::RegisterX, &Args::StackPointer),
      0xFA => self.mov(&Args::DirectPage, &Args::DirectPage),
      0xD4 => self.mov(&Args::Direct_Page_Indexed_by_X, &Args::RegisterA),

      0xD6 => self.mov(&Args::Absolute_Indexed_by_Y, &Args::RegisterA), // MOV !abs+Y, A
      0xD8 => self.mov(&Args::DirectPage, &Args::RegisterX), // MOV dp, X
      0xD9 => self.mov(&Args::Direct_Page_Indexed_by_Y, &Args::RegisterX), // MOV dp+Y, X
      0xC9 => self.mov(&Args::Absolute, &Args::RegisterX), // MOV !abs, X
      0xDB => self.mov(&Args::Direct_Page_Indexed_by_X, &Args::RegisterY), // MOV dp+X, Y
      0xCC => self.mov(&Args::Absolute, &Args::RegisterY), // MOV !abs, Y
      0x3E => self.cmp(&Args::RegisterX, &Args::DirectPage),

      0xE6 => self.mov(&Args::RegisterA, &Args::RegisterX_Direct_Page_Indirect), // MOV A, (X)
      0xBF => self.mov(&Args::RegisterA, &Args::RegisterX_Direct_Page_Indirect_Incriment), // MOV A, (X)+
      0xF4 => self.mov(&Args::RegisterA, &Args::Direct_Page_Indexed_by_X), // MOV A, dp+X
      0xF6 => self.mov(&Args::RegisterA, &Args::Absolute_Indexed_by_Y), // MOV A, !abs+Y
      0xE7 => self.mov(&Args::RegisterA, &Args::Direct_Page_Indexed_Indirect_by_X), // MOV A, [dp+X]
      0xF7 => self.mov(&Args::RegisterA, &Args::Direct_Page_Indirect_Indexed_by_Y), // MOV A, [dp]+Y
      0xF8 => self.mov(&Args::RegisterX, &Args::DirectPage), // MOV X, dp
      0xF9 => self.mov(&Args::RegisterX, &Args::Direct_Page_Indexed_by_Y), // MOV X, dp+Y
      0xE9 => self.mov(&Args::RegisterX, &Args::Absolute), // MOV X, !abs
      0xFB => self.mov(&Args::RegisterY, &Args::Direct_Page_Indexed_by_X), // MOV Y, dp+X
      0x7A => self.addw(&Args::RegisterYA, &Args::DirectPage),
      0xCE => self.pop(&Args::RegisterX),
      0xEE => self.pop(&Args::RegisterY),
      0x8E => self.pop(&Args::ProgramStatus),
      0x0B => self.asl(&Args::DirectPage), // FIXME pswがダメ
      _ => panic!("not implement op: {:02X}", op)
    }
    */
    call(self, op);
    let cycles = get_cycles(op);
    // self.cycles += cycles;
  }

  fn tclr1(&mut self, mode: &Args) {
    let value = self._get_value_without_inc_pc(mode);
    let a = self.get_register_a();
    let v = value & !a;
    self._set_value(mode, v);
    self._set_status(FLAG_NEGATIVE, (a.wrapping_sub(value) & 0x80) != 0);
    self._set_status(FLAG_ZERO, a == value);
  }

  fn tset1(&mut self, mode: &Args) {
    let value = self._get_value_without_inc_pc(mode);
    let a = self.get_register_a();
    let v = value | a;
    self._set_value(mode, v);
    self._set_status(FLAG_NEGATIVE, (a.wrapping_sub(value) & 0x80) != 0);
    self._set_status(FLAG_ZERO, a == value);

  }

  fn pcall(&mut self) {
    let addr = self.mem_read(self.program_counter) as u16;
    self.inc_program_counter();

    let addr = 0xFF00 | addr;

    self._push_u16(self.program_counter);
    self.program_counter = addr;
  }

  fn tcall(&mut self, op: u8) {
    let index = (op & 0xF0) >> 4;
    let addr = 0xFFDE - index as u16 * 2;
    let addr = self.mem_read_u16(addr);
    self._push_u16(self.program_counter);
    self.program_counter = addr;
  }

  fn bbs(&mut self, op: u8) {
    let bit = (op & 0xE0) >> 5;
    let mask = 0x01 << bit;
    let value = self._get_value(&Args::DirectPage);
    self._jump((value & mask) != 0);
  }

  fn bbc(&mut self, op: u8) {
    let bit = (op & 0xE0) >> 5;
    let mask = 0x01 << bit;
    let value = self._get_value(&Args::DirectPage);
    self._jump((value & mask) == 0);
  }

  fn cbne(&mut self, mode: &Args) {
    let value = self._get_value(mode);
    let addr = (self._get_value(&Args::Immediate) as i8) as u16;
    if value != self.get_register_a() {
      // ジャンプ
      self.program_counter = self.program_counter.wrapping_add(addr);
    }
  }
  fn dbnz(&mut self, mode: &Args) {
    let value = self._get_value_without_inc_pc(mode);
    let value = value.wrapping_sub(1);
    self._set_value(mode, value);
    let jump = value != 0;

    let addr = (self._get_value(&Args::Immediate) as i8) as u16;
    if jump {
      // ジャンプ
      self.program_counter = self.program_counter.wrapping_add(addr);
    }
  }

  fn bvs(&mut self) {
    self._jump((self.program_status & FLAG_OVERFLOW) != 0)
  }

  fn bvc(&mut self) {
    self._jump((self.program_status & FLAG_OVERFLOW) == 0)
  }

  fn or(&mut self, left: &Args, right: &Args) {
    let right = self._get_value(right);
    let left_value = self._get_value_without_inc_pc(left);

    let value = left_value | right;
    self._set_value(left, value);
    self.update_negative_and_zero_flags(value);
  }

  fn or1(&mut self, op: u8) {
    let rev = op == 0x2A;
    let operand = self._get_value_u16(&Args::Immediate);
    let addr = operand & 0x1FFF;
    let bit = (operand & 0xE000) >> 13;
    let mask = 0x01 << bit;
    let value = self.mem_read(addr);
    let value = if rev {
      !value
    } else {
      value
    };
    let v = (value & mask) >> bit;
    let carry = self.program_status & FLAG_CARRY;
    let v = (v | carry) != 0;
    self._set_status(FLAG_CARRY, v);
  }

  fn and1(&mut self, op: u8) {
    let rev = op == 0x6A;
    let operand = self._get_value_u16(&Args::Immediate);
    let addr = operand & 0x1FFF;
    let bit = (operand & 0xE000) >> 13;
    let mask = 0x01 << bit;
    let value = self.mem_read(addr);
    let value = if rev {
      !value
    } else {
      value
    };
    let v = (value & mask) >> bit;
    let carry = self.program_status & FLAG_CARRY;
    let v = (v & carry) != 0;
    self._set_status(FLAG_CARRY, v);
  }

  fn eor1(&mut self) {
    let operand = self._get_value_u16(&Args::Immediate);
    let addr = operand & 0x1FFF;
    let bit = (operand & 0xE000) >> 13;
    let mask = 0x01 << bit;
    let value = self.mem_read(addr);
    let v = (value & mask) >> bit;
    let carry = self.program_status & FLAG_CARRY;
    let v = (v ^ carry) != 0;
    self._set_status(FLAG_CARRY, v);
  }

  fn not1(&mut self, op: u8, mode: &Args) {}
  fn mov1(&mut self, dest: &Args, src: &Args, src2: &Args) {}
  fn brk(&mut self) {}
  fn decw(&mut self, mode: &Args) {}
  fn lsr(&mut self, mode: &Args) {}
  fn rol(&mut self, mode: &Args) {}
  fn ror(&mut self, mode: &Args) {}
  fn reti(&mut self) {}
  fn div(&mut self) {}
  fn xcn(&mut self) {}
  fn ei(&mut self) {}
  fn das(&mut self) {}
  fn daa(&mut self) {}
  fn clrv(&mut self) {}
  fn notc(&mut self) {}
  fn sleep(&mut self) {}
  fn stop(&mut self) {}

  fn addw(&mut self, dest: &Args, src: &Args) {
    let value1 = self._get_value_u16(src);
    let value2 = self._get_value_u16_without_inc_pc(dest);
    let (value, carry) = value2.overflowing_add(value1);

    let overflow = (value2 & SIGN_BIT_U16) == (value1 & SIGN_BIT_U16)
        && (value1 & SIGN_BIT_U16) != (value & SIGN_BIT_U16);
    let half_carry = (((value2 & 0x0FFF) + (value1 & 0x0FFF)) & 0x1000) != 0;

    self._set_status(FLAG_OVERFLOW, overflow);
    self._set_status(FLAG_CARRY, carry);
    self._set_status(FLAG_HALF_CARRY, half_carry);
    self._set_value_u16(dest, value);
  }

  fn mul(&mut self) {
    let a = self.get_register_a() as u16;
    let y = self.get_register_y() as u16;
    self.ya = a * y;
    self.update_negative_and_zero_flags(self.get_register_y());
  }

  fn set1(&mut self, op: u8, mode: &Args) {
    let value = self._get_value_without_inc_pc(&Args::DirectPage);

    // オペコードの最上位3ビットによって、そのバイト内のどのビットが設定されるかが決まります。
    let bit = (op & 0xE0) >> 5;
    let value = value | (0x01 << bit);

    self._set_value(&Args::DirectPage, value);
  }

  fn clr1(&mut self, op: u8, mode: &Args) {
    let value = self._get_value_without_inc_pc(&Args::DirectPage);

    // オペコードの最上位3ビットによって、そのバイト内のどのビットがクリアされるかが決まります。
    let bit = (op & 0xE0) >> 5;
    let value = value & !(0x01 << bit);

    self._set_value(&Args::DirectPage, value);
  }

  fn asl(&mut self, mode: &Args) {
    let value = self._get_value_without_inc_pc(mode);
    let carry = (value & 0x80) != 0;
    let value = value << 1;
    self._set_value(mode, value);
    self._set_status(FLAG_CARRY, carry);
    // DirectPageの時も更新が必要。
    self.update_negative_and_zero_flags(value);
  }

  fn _set_status(&mut self, flag: u8, value: bool) {
    self.program_status = if value {
        self.program_status | flag
    } else {
        self.program_status & !flag
    };
  }

  fn pop(&mut self, mode: &Args) {
    let value = self._pop();
    self._set_value_without_update_status(mode, value);
  }

  fn push(&mut self, mode: &Args) {
    let value = self._get_value(mode);
    self._push(value);
  }

  fn and(&mut self, left: &Args, right: &Args) {
    let right = self._get_value(right);
    let left_value = self._get_value_without_inc_pc(left);

    let value = left_value & right;
    self._set_value(left, value);
  }

  fn eor(&mut self, left: &Args, right: &Args) {
    let right = self._get_value(right);
    let left_value = self._get_value_without_inc_pc(left);

    let value = left_value ^ right;
    self._set_value(left, value);
  }

  fn subw(&mut self, dest: &Args, src: &Args) {
    let value1 = self._get_value_u16(src);
    let value2 = self._get_value_u16_without_inc_pc(dest);
    let (value, carry) = value2.overflowing_sub(value1);

    let overflow = (value2 & SIGN_BIT_U16) != (value1 & SIGN_BIT_U16) && (value2 & SIGN_BIT_U16) != (value & SIGN_BIT_U16);

    // println!("SUBW {:04X} - {:04X} => {:04X}", value2, value1, value);
    let half_carry = (value2 & 0x0FFF) < (value1 & 0x0FFF);

    self._set_status(FLAG_CARRY, !carry);
    self._set_status(FLAG_OVERFLOW, overflow);
    self._set_status(FLAG_HALF_CARRY, !half_carry);
    self._set_value_u16(dest, value);
  }

  fn sbc(&mut self, dest: &Args, src: &Args) {
    let value1 = self._get_value(src);
    let value2 = self._get_value_without_inc_pc(dest);

    let carry = 1 - (self.program_status & FLAG_CARRY);
    let (rhs, carry_flag1) = value2.overflowing_sub(value1);
    let (n, carry_flag2) = rhs.overflowing_sub(carry);

    let overflow = (value2 & SIGN_BIT) != (value1 & SIGN_BIT)
        && (value2 & SIGN_BIT) != (n & SIGN_BIT);

    let (rhs, half_carry1) = (value2 & 0x0F).overflowing_sub(value1 & 0x0F);
    let (_, half_carry2) = rhs.overflowing_sub(carry);

    self._set_value(dest, n);
    self._set_status(FLAG_CARRY, !carry_flag1 && !carry_flag2);
    self._set_status(FLAG_OVERFLOW, overflow);
    self._set_status(FLAG_HALF_CARRY, !half_carry1 && !half_carry2);
  }

  fn adc(&mut self, dest: &Args, src: &Args) {
    let value1 = self._get_value(src);
    let value2 = self._get_value_without_inc_pc(dest);

    let carry = self.program_status & FLAG_CARRY;
    let (rhs, carry_flag1) = value1.overflowing_add(carry);
    let (n, carry_flag2) = value2.overflowing_add(rhs);

    let overflow = (value2 & SIGN_BIT) == (value1 & SIGN_BIT)
        && (value1 & SIGN_BIT) != (n & SIGN_BIT);

    let half_carry = (((value1 & 0x0F) + (value2 & 0x0F) + carry) & 0x10) != 0;

    self._set_value(dest, n);
    self._set_status(FLAG_CARRY, carry_flag1 || carry_flag2);
    self._set_status(FLAG_OVERFLOW, overflow);
    self._set_status(FLAG_HALF_CARRY, half_carry);
  }

  // CMPW YA, dp
  fn cmpw(&mut self, left: &Args, right: &Args) {
    let right = self._get_value_u16(right);
    let left = self._get_value_u16(left);
    let (value, carry) = left.overflowing_sub(right);
    self._set_status(FLAG_CARRY, !carry);
    self.update_negative_and_zero_flags_u16(value);
  }

  fn incw(&mut self, mode: &Args) {
    let addr = self.mem_read(self.program_counter);
    self.inc_program_counter();
    let addr = self.direct_page_addr(addr);
    let data = self.mem_read_u16(addr);
    let data = data.wrapping_add(1);
    self.mem_write_u16(addr, data);
    self.update_negative_and_zero_flags_u16(data);
  }

  fn setc(&mut self) {
    self.program_status = self.program_status | FLAG_CARRY;
  }

  fn setp(&mut self) {
    self.program_status = self.program_status | FLAG_DIRECT_PAGE;
  }

  fn clrp(&mut self) {
    self.program_status = self.program_status & !FLAG_DIRECT_PAGE;
  }

  fn clrc(&mut self) {
    self.program_status = self.program_status & !FLAG_CARRY;
  }

  fn call(&mut self, mode: &Args) {
    let addr = self.mem_read_u16_no_wrapped(self.program_counter);
    self.inc_program_counter();
    self.inc_program_counter();

    let lo = (self.program_counter & 0x00FF) as u8;
    let hi = ((self.program_counter & 0xFF00) >> 8) as u8;
    self._push(hi);
    self._push(lo);
    self.program_counter = addr;
  }

  fn ret(&mut self) {
    let lo = self._pop() as u16;
    let hi = self._pop() as u16;
    self.program_counter = hi << 8 | lo;
  }

  fn di(&mut self) {
    self.program_status = self.program_status & !FLAG_INTERRRUPT;
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

  fn _get_value_without_inc_pc(&mut self, arg: &Args) -> u8 {
    self._disable_inc_pc = true;
    let value = self._get_value(arg);
    self._disable_inc_pc = false;
    value
  }

  fn _set_value_without_update_status(&mut self, arg: &Args, data: u8) {
    self._disable_update_status = true;
    self._set_value(arg, data);
    self._disable_update_status = false;
  }

  fn _get_value(&mut self, arg: &Args) -> u8 {
    match arg {
      Args::RegisterY => {
        self.get_register_y()
      }
      Args::RegisterX => {
        self.get_register_x()
      }
      Args::RegisterA => {
        self.get_register_a()
      }
      Args::Absolute_Indexed_by_X => {
        let addr = self.mem_read_u16_no_wrapped(self.program_counter);
        self.inc_program_counter();
        self.inc_program_counter();
        let addr = addr.wrapping_add(self.get_register_x() as u16);
        let data = self.mem_read(addr);
        data
      }
      Args::Immediate => {
        let data = self.mem_read(self.program_counter);
        self.inc_program_counter();
        data
      }
      Args::DirectPage => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let data = self.mem_read(self.direct_page_addr(addr));
        data
      }
      Args::Absolute => {
        let addr = self.mem_read_u16_no_wrapped(self.program_counter);
        self.inc_program_counter();
        self.inc_program_counter();
        let data = self.mem_read(addr);
        data
      }
      Args::StackPointer => {
        self.stack_pointer
      }
      Args::RegisterX_Direct_Page_Indirect => {
        let addr = self.direct_page_addr(self.get_register_x());
        let data = self.mem_read(addr);
        data
      }
      Args::RegisterX_Direct_Page_Indirect_Incriment => {
        let addr = self.direct_page_addr(self.get_register_x());
        let data = self.mem_read(addr);
        self.set_register_x(self.get_register_x().wrapping_add(1));
        data
      }
      Args::RegisterY_Direct_Page_Indirect => {
        let addr = self.direct_page_addr(self.get_register_y());
        let data = self.mem_read(addr);
        data
      }
      Args::Direct_Page_Indexed_by_X => {
        let addr = self.mem_read(self.program_counter);
        let x = self.get_register_x();
        let addr = addr.wrapping_add(x);
        let addr = self.direct_page_addr(addr);
        self.inc_program_counter();
        let data = self.mem_read(addr);
        data
      }
      Args::Absolute_Indexed_by_Y => {
        let addr = self.mem_read_u16_no_wrapped(self.program_counter);
        self.inc_program_counter();
        self.inc_program_counter();
        let addr = addr.wrapping_add(self.get_register_y() as u16);
        let data = self.mem_read(addr);
        data
      }
      Args::Direct_Page_Indexed_Indirect_by_X => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let addr = addr.wrapping_add(self.get_register_x());
        let addr = self.direct_page_addr(addr);
        let addr = self.mem_read_u16(addr);
        let data = self.mem_read(addr);
        data
      }
      Args::Direct_Page_Indirect_Indexed_by_Y => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let addr = self.direct_page_addr(addr);
        let addr = self.mem_read_u16(addr) as u16;
        let addr = addr.wrapping_add(self.get_register_y() as u16);
        let data = self.mem_read(addr);
        data
      }
      Args::Direct_Page_Indexed_by_Y => {
        let addr = self.mem_read(self.program_counter);
        let y = self.get_register_y();
        let addr = addr.wrapping_add(y);
        let addr = self.direct_page_addr(addr);
        self.inc_program_counter();
        let data = self.mem_read(addr);
        data
      }
      _ => panic!("not implemented _get_value")
    }
  }

  fn _set_value(&mut self, arg: &Args, data: u8) {
    match arg {
      Args::RegisterA => {
        self.set_register_a(data);
        self.update_negative_and_zero_flags(data);
      }
      Args::RegisterX => {
        self.set_register_x(data);
        self.update_negative_and_zero_flags(data);
      }
      Args::RegisterY => {
        self.set_register_y(data);
        self.update_negative_and_zero_flags(data);
      }
      // MOV "(X)+", A
      Args::RegisterX_Direct_Page_Indirect_Incriment => {
        let addr = self.direct_page_addr(self.get_register_x());
        self.mem_write(addr, data);
        self.set_register_x(self.get_register_x().wrapping_add(1));
      }
      // MOV "(X)",A
      Args::RegisterX_Direct_Page_Indirect => {
        let addr = self.direct_page_addr(self.get_register_x());
        self.mem_write(addr, data);
      }
      // MOV "[$00]+Y",A
      Args::Direct_Page_Indirect_Indexed_by_Y => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let addr = self.direct_page_addr(addr);
        let addr = self.mem_read_u16(addr) as u16;
        let addr = addr.wrapping_add(self.get_register_y() as u16);
        self.mem_write(addr, data);
      }
      // //MOV [dp+X], A
      Args::Direct_Page_Indexed_Indirect_by_X => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let addr = addr.wrapping_add(self.get_register_x());
        let addr = self.direct_page_addr(addr);
        let addr = self.mem_read_u16(addr) as u16;
        self.mem_write(addr, data);
      }
      Args::StackPointer => {
        self.set_stack_pointer(data);
      }
      Args::DirectPage => {
        let dest = self.mem_read(self.program_counter);
        let dest = self.direct_page_addr(dest);
        self.inc_program_counter();
        self.mem_write(dest as u16, data);
      }
      Args::Absolute => {
        let addr = self.mem_read_u16_no_wrapped(self.program_counter);
        self.inc_program_counter();
        self.inc_program_counter();
        self.mem_write(addr, data);
      }
      Args::Absolute_Indexed_by_X => {
        let addr = self.mem_read_u16_no_wrapped(self.program_counter);
        self.inc_program_counter();
        self.inc_program_counter();
        let addr = addr.wrapping_add(self.get_register_x() as u16);
        self.mem_write(addr, data);
      }
      Args::Direct_Page_Indexed_by_X => {
        let dest = self.mem_read(self.program_counter);
        let x = self.get_register_x();
        let dest = dest.wrapping_add(x);
        let dest = self.direct_page_addr(dest);
        self.inc_program_counter();
        self.mem_write(dest as u16, data);
      }
      Args::Absolute_Indexed_by_Y => {
        let addr = self.mem_read_u16_no_wrapped(self.program_counter);
        self.inc_program_counter();
        self.inc_program_counter();
        let addr = addr.wrapping_add(self.get_register_y() as u16);
        self.mem_write(addr, data);
      }
      Args::Direct_Page_Indexed_by_Y => {
        let dest = self.mem_read(self.program_counter);
        let y = self.get_register_y();
        let dest = dest.wrapping_add(y);
        let dest = self.direct_page_addr(dest);
        self.inc_program_counter();
        self.mem_write(dest as u16, data);
      }
      Args::ProgramStatus => {
        self.program_status = data
      }
      _ => panic!("not implemented _set_value")
    }
  }

  fn _get_value_u16_without_inc_pc(&mut self, arg: &Args) -> u16 {
    self._disable_inc_pc = true;
    let value = self._get_value_u16(arg);
    self._disable_inc_pc = false;
    value
  }

  fn _get_value_u16(&mut self, arg: &Args) -> u16 {
    match arg {
      Args::Immediate => {
        let v = self.mem_read_u16_no_wrapped(self.program_counter);
        self.inc_program_counter();
        self.inc_program_counter();
        v
      }
      Args::RegisterYA => {
        self.ya
      }
      Args::DirectPage => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let data = self.mem_read_u16(self.direct_page_addr(addr));
        data
      }
      _ => panic!("not implemented _get_value_u16")
    }
  }

  fn _set_value_u16(&mut self, arg: &Args, data: u16) {
    match arg {
      Args::RegisterYA => {
        self.ya = data;
        self.update_negative_and_zero_flags_u16(data);
      }
      _ => panic!("not implemented _set_value_u16")
    }
  }

  // MOV A,Y
  // MOV X,A
  fn mov(&mut self, dest: &Args, src: &Args) {
    let data = self._get_value(src);
    self._set_value(dest, data);
  }

  // MOVW YA,$F6
  fn movw(&mut self, dest: &Args, src: &Args) {
    let data = match src {
      Args::DirectPage => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let addr = self.direct_page_addr(addr);
        let data = self.mem_read_u16(addr);
        data
      }
      Args::RegisterYA => {
        self.ya
      }
      _ => panic!("not implemented movw")
    };
    match dest {
      Args::RegisterYA => {
        self.ya = data;
        self.update_negative_and_zero_flags_u16(data);
      }
      Args::DirectPage => {
        let addr = self.mem_read(self.program_counter);
        self.inc_program_counter();
        let addr = self.direct_page_addr(addr);
        self.mem_write_u16(addr, data);
      }
      _ => panic!("not implemented movw")
    }
  }

  // DEC X
  fn dec(&mut self, mode: &Args) {
    match mode {
      Args::RegisterX => {
        let x = self.get_register_x().wrapping_sub(1);
        self.set_register_x(x);
        self.update_negative_and_zero_flags(x);
      }
      _ => panic!("not implemented dec")
    }
  }

  // INC Y
  // INC $01
  fn inc(&mut self, mode: &Args) {
    match mode {
      Args::RegisterA => {
        let a = self.get_register_a().wrapping_add(1);
        self.set_register_a(a);
        self.update_negative_and_zero_flags(a);
      }
      Args::RegisterY => {
        let y = self.get_register_y().wrapping_add(1);
        self.set_register_y(y);
        self.update_negative_and_zero_flags(y);
      }
      Args::RegisterX => {
        let x = self.get_register_x().wrapping_add(1);
        self.set_register_x(x);
        self.update_negative_and_zero_flags(x);
      }
      Args::DirectPage => {
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

  fn bcs(&mut self) {
    self._jump((self.program_status & FLAG_CARRY) != 0);
  }

  fn bmi(&mut self) {
    self._jump((self.program_status & FLAG_NEGATIVE) != 0);
  }

  // BNE $FC
  fn bne(&mut self) {
    self._jump((self.program_status & FLAG_ZERO) == 0);
  }

  // BRA Start
  fn bra(&mut self) {
    self._jump(true);
  }

  // BPL -
  fn bpl(&mut self) {
    self._jump((self.program_status & FLAG_NEGATIVE) == 0);
  }

  fn beq(&mut self) {
    self._jump((self.program_status & FLAG_ZERO) != 0);
  }

  fn bcc(&mut self) {
    self._jump((self.program_status & FLAG_CARRY) == 0);
  }

  fn _jump(&mut self, jump: bool) {
    let v = self.mem_read(self.program_counter) as i8;
    let v = v as i16;
    let v = v as u16;
    self.inc_program_counter();
    if !jump {
      return
    }
    self.program_counter = self.program_counter.wrapping_add(v)
  }

  fn cmp(&mut self, left: &Args, right: &Args) {
    // CMP X, dp
    let right = self._get_value(right);
    let left = self._get_value(left);
    let (value, carry) = left.overflowing_sub(right);
    self._set_status(FLAG_CARRY, !carry);
    self.update_negative_and_zero_flags(value);
  }

  // JMP [$0000+X]
  fn jmp(&mut self, mode: &Args) {
    let addr = match mode {
      Args::Absolute_Indexed_by_X => {
        let addr = self.mem_read_u16_no_wrapped(self.program_counter);
        let x = self.get_register_x();
        let addr = addr.wrapping_add(x as u16);
        let addr = self.mem_read_u16_no_wrapped(addr);
        addr
      }
      Args::Absolute => {
        let addr = self.mem_read_u16_no_wrapped(self.program_counter);
        addr
      }
      _ => panic!("not implemented jmp")
    };
    self.program_counter = addr;
  }

  fn inc_program_counter(&mut self) {
    if self._disable_inc_pc {
      return
    }
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
    if self._disable_update_status {
      return
    }

    let test_bit = 0x80;
    self._set_status(FLAG_ZERO, result == 0);
    self._set_status(FLAG_NEGATIVE, (result & test_bit) != 0);
  }

  fn update_negative_and_zero_flags_u16(&mut self, result: u16) {
    if self._disable_update_status {
      return
    }

    let test_bit = 0x8000;
    self._set_status(FLAG_ZERO, result == 0);
    self._set_status(FLAG_NEGATIVE, (result & test_bit) != 0);
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

  pub fn get_stack_pointer(&self) -> u16 {
    0x0100 | self.stack_pointer as u16
  }

  pub fn set_stack_pointer(&mut self, value: u8) {
    self.stack_pointer = value
  }

  fn _push_u16(&mut self, value: u16) {
    let lo = value as u8;
    let hi = ((value & 0xFF00) >> 8) as u8;
    self._push(hi);
    self._push(lo);
  }

  fn _push(&mut self, value: u8) {
    self.mem_write(self.get_stack_pointer(), value);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
  }

  fn _pop(&mut self) -> u8 {
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    let addr = self.get_stack_pointer();
    let value = self.mem_read(addr);
    value
  }

  pub fn mem_read(&mut self, addr: u16) -> u8 {
    if self.unit_test_mode {
      return self.memory[addr as usize];
    }

    match addr {
      0x00F4..=0x00F7 => {
        let port = (addr - 0x00F4) % 4;
        self.input[port as usize]
      }
      _ => self.memory[addr as usize]
    }
  }

  pub fn mem_write(&mut self, addr: u16, data: u8) {
    if self.unit_test_mode {
      self.memory[addr as usize] = data;
      return
    }

    match addr {
      0x00F0 => {
        // 0x00F0	TEST	テスト機能
      }
      0x00F1 => {
        // 0x00F1	CONTROL	I/Oとタイマーの調整
        if (data & 0x10) != 0 {
          // 1 を書き込んだ時、入力ポートの 0x00F4 と 0x00F5 が 0x00 にクリアされます。
          //self.mem_write(0x00F4, 0);
          //self.mem_write(0x00F5, 0);
        }
        if (data & 0x20) != 0 {
          // 1 を書き込んだ時、入力ポートの 0x00F6 と 0x00F7 が 0x00 にクリアされます。
          //self.mem_write(0x00F6, 0);
          //self.mem_write(0x00F7, 0);
        }
      }
      0x00F2..=0x00F3 => {

        // 0x00F2	DSPADDR	DSP通信アドレス
        // 0x00F3	DSPDATA	DSP通信データ
        println!("APU mem_write({:04X}, {:02X})", addr, data)
      }
      0x00F4..=0x00F7 => {
        let port = (addr - 0x00F4) % 4;
        self.output[port as usize] = data
      }
      0x00FA..=0x00FF => {
        // タイマー関連
        println!("APU mem_write({:04X}, {:02X})", addr, data)
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

  pub fn mem_write_u16_no_wrapped(&mut self, addr: u16, data: u16) {
    self.mem_write(addr, (data & 0x00FF) as u8);
    let addr = addr.wrapping_add(1);
    self.mem_write(addr, (data >> 8) as u8);
  }

  pub fn write(&mut self, addr: u16, data: u8) {
    println!("APU write FROM CPU({:04X}, {:02X})", addr, data);
    match addr {
      0x2140..=0x217F => {
        let port = (addr - 0x2140) % 4;
        self.input[port as usize] = data;
      }
      _ => panic!("should not reach. APU::write"),
    }
  }

  pub fn read(&mut self, addr: u16) -> u8 {
    println!("APU read FROM CPU({:04X})", addr);
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

// GENERATED by scraping/apu/generate_code.ts
fn get_cycles(op: u8) -> u8 {
  match op {
    0x00 => 2,
    0x01 => 8,
    0x02 => 4,
    0x03 => 7,
    0x04 => 3,
    0x05 => 4,
    0x06 => 3,
    0x07 => 6,
    0x08 => 2,
    0x09 => 6,
    0x0A => 5,
    0x0B => 4,
    0x0C => 5,
    0x0D => 4,
    0x0E => 6,
    0x0F => 8,
    0x10 => 4,
    0x11 => 8,
    0x12 => 4,
    0x13 => 7,
    0x14 => 4,
    0x15 => 5,
    0x16 => 5,
    0x17 => 6,
    0x18 => 5,
    0x19 => 5,
    0x1A => 6,
    0x1B => 5,
    0x1C => 2,
    0x1D => 2,
    0x1E => 4,
    0x1F => 6,
    0x20 => 2,
    0x21 => 8,
    0x22 => 4,
    0x23 => 7,
    0x24 => 3,
    0x25 => 4,
    0x26 => 3,
    0x27 => 6,
    0x28 => 2,
    0x29 => 6,
    0x2A => 5,
    0x2B => 4,
    0x2C => 5,
    0x2D => 4,
    0x2E => 7,
    0x2F => 4,
    0x30 => 4,
    0x31 => 8,
    0x32 => 4,
    0x33 => 7,
    0x34 => 4,
    0x35 => 5,
    0x36 => 5,
    0x37 => 6,
    0x38 => 5,
    0x39 => 5,
    0x3A => 6,
    0x3B => 5,
    0x3C => 2,
    0x3D => 2,
    0x3E => 3,
    0x3F => 8,
    0x40 => 2,
    0x41 => 8,
    0x42 => 4,
    0x43 => 7,
    0x44 => 3,
    0x45 => 4,
    0x46 => 3,
    0x47 => 6,
    0x48 => 2,
    0x49 => 6,
    0x4A => 4,
    0x4B => 4,
    0x4C => 5,
    0x4D => 4,
    0x4E => 6,
    0x4F => 6,
    0x50 => 4,
    0x51 => 8,
    0x52 => 4,
    0x53 => 7,
    0x54 => 4,
    0x55 => 5,
    0x56 => 5,
    0x57 => 6,
    0x58 => 5,
    0x59 => 5,
    0x5A => 4,
    0x5B => 5,
    0x5C => 2,
    0x5D => 2,
    0x5E => 4,
    0x5F => 3,
    0x60 => 2,
    0x61 => 8,
    0x62 => 4,
    0x63 => 7,
    0x64 => 3,
    0x65 => 4,
    0x66 => 3,
    0x67 => 6,
    0x68 => 2,
    0x69 => 6,
    0x6A => 4,
    0x6B => 4,
    0x6C => 5,
    0x6D => 4,
    0x6E => 7,
    0x6F => 5,
    0x70 => 4,
    0x71 => 8,
    0x72 => 4,
    0x73 => 7,
    0x74 => 4,
    0x75 => 5,
    0x76 => 5,
    0x77 => 6,
    0x78 => 5,
    0x79 => 5,
    0x7A => 5,
    0x7B => 5,
    0x7C => 2,
    0x7D => 2,
    0x7E => 3,
    0x7F => 6,
    0x80 => 2,
    0x81 => 8,
    0x82 => 4,
    0x83 => 7,
    0x84 => 3,
    0x85 => 4,
    0x86 => 3,
    0x87 => 6,
    0x88 => 2,
    0x89 => 6,
    0x8A => 5,
    0x8B => 4,
    0x8C => 5,
    0x8D => 2,
    0x8E => 4,
    0x8F => 5,
    0x90 => 4,
    0x91 => 8,
    0x92 => 4,
    0x93 => 7,
    0x94 => 3,
    0x95 => 5,
    0x96 => 5,
    0x97 => 6,
    0x98 => 5,
    0x99 => 5,
    0x9A => 5,
    0x9B => 5,
    0x9C => 2,
    0x9D => 2,
    0x9E => 2,
    0x9F => 5,
    0xA0 => 3,
    0xA1 => 8,
    0xA2 => 4,
    0xA3 => 7,
    0xA4 => 3,
    0xA5 => 4,
    0xA6 => 3,
    0xA7 => 6,
    0xA8 => 2,
    0xA9 => 6,
    0xAA => 4,
    0xAB => 4,
    0xAC => 5,
    0xAD => 2,
    0xAE => 4,
    0xAF => 4,
    0xB0 => 4,
    0xB1 => 8,
    0xB2 => 4,
    0xB3 => 7,
    0xB4 => 4,
    0xB5 => 5,
    0xB6 => 5,
    0xB7 => 6,
    0xB8 => 5,
    0xB9 => 5,
    0xBA => 5,
    0xBB => 5,
    0xBC => 2,
    0xBD => 2,
    0xBE => 3,
    0xBF => 4,
    0xC0 => 3,
    0xC1 => 8,
    0xC2 => 4,
    0xC3 => 7,
    0xC4 => 4,
    0xC5 => 5,
    0xC6 => 4,
    0xC7 => 7,
    0xC8 => 2,
    0xC9 => 5,
    0xCA => 6,
    0xCB => 4,
    0xCC => 5,
    0xCD => 2,
    0xCE => 4,
    0xCF => 9,
    0xD0 => 4,
    0xD1 => 8,
    0xD2 => 4,
    0xD3 => 7,
    0xD4 => 5,
    0xD5 => 6,
    0xD6 => 6,
    0xD7 => 6,
    0xD8 => 4,
    0xD9 => 5,
    0xDA => 4,
    0xDB => 5,
    0xDC => 2,
    0xDD => 2,
    0xDE => 8,
    0xDF => 3,
    0xE0 => 2,
    0xE1 => 8,
    0xE2 => 4,
    0xE3 => 7,
    0xE4 => 3,
    0xE5 => 4,
    0xE6 => 3,
    0xE7 => 6,
    0xE8 => 3,
    0xE9 => 4,
    0xEA => 5,
    0xEB => 3,
    0xEC => 4,
    0xED => 3,
    0xEE => 4,
    0xEF => 3,
    0xF0 => 4,
    0xF1 => 8,
    0xF2 => 4,
    0xF3 => 7,
    0xF4 => 4,
    0xF5 => 5,
    0xF6 => 5,
    0xF7 => 6,
    0xF8 => 3,
    0xF9 => 4,
    0xFA => 5,
    0xFB => 4,
    0xFC => 2,
    0xFD => 2,
    0xFE => 6,
    0xFF => 3,
  }
}

fn call(cpu: &mut APU, op: u8) {
  match op {
    0x00 => cpu.nop(),
    0x01 => cpu.tcall(op),
    0x02 => cpu.set1(op, &Args::bit/* bit */),
    0x03 => cpu.bbs(op),
    0x04 => cpu.or(&Args::RegisterA, &Args::DirectPage),
    0x05 => cpu.or(&Args::RegisterA, &Args::Absolute),
    0x06 => cpu.or(&Args::RegisterA, &Args::RegisterX_Direct_Page_Indirect),
    0x07 => cpu.or(&Args::RegisterA, &Args::Direct_Page_Indexed_Indirect_by_X),
    0x08 => cpu.or(&Args::RegisterA, &Args::Immediate),
    0x09 => cpu.or(&Args::DirectPage, &Args::DirectPage),
    0x0A => cpu.or1(op),
    0x0B => cpu.asl(&Args::DirectPage),
    0x0C => cpu.asl(&Args::Absolute),
    0x0D => cpu.push(&Args::ProgramStatus),
    0x0E => cpu.tset1(&Args::Absolute),
    0x0F => cpu.brk(),
    0x10 => cpu.bpl(),
    0x11 => cpu.tcall(op),
    0x12 => cpu.clr1(op, &Args::bit/* bit */),
    0x13 => cpu.bbc(op),
    0x14 => cpu.or(&Args::RegisterA, &Args::Direct_Page_Indexed_by_X),
    0x15 => cpu.or(&Args::RegisterA, &Args::Absolute_Indexed_by_X),
    0x16 => cpu.or(&Args::RegisterA, &Args::Absolute_Indexed_by_Y),
    0x17 => cpu.or(&Args::RegisterA, &Args::Direct_Page_Indirect_Indexed_by_Y),
    0x18 => cpu.or(&Args::DirectPage, &Args::Immediate),
    0x19 => cpu.or(&Args::RegisterX_Direct_Page_Indirect, &Args::RegisterY_Direct_Page_Indirect),
    0x1A => cpu.decw(&Args::DirectPage),
    0x1B => cpu.asl(&Args::dpx/* dp+x */),
    0x1C => cpu.asl(&Args::RegisterA),
    0x1D => cpu.dec(&Args::RegisterX),
    0x1E => cpu.cmp(&Args::RegisterY, &Args::Immediate),
    0x1F => cpu.jmp(&Args::Absolute_Indexed_by_X),
    0x20 => cpu.clrp(),
    0x21 => cpu.tcall(op),
    0x22 => cpu.set1(op, &Args::bit/* bit */),
    0x23 => cpu.bbs(op),
    0x24 => cpu.and(&Args::RegisterA, &Args::DirectPage),
    0x25 => cpu.and(&Args::RegisterA, &Args::Absolute),
    0x26 => cpu.and(&Args::RegisterA, &Args::RegisterX_Direct_Page_Indirect),
    0x27 => cpu.and(&Args::RegisterA, &Args::Direct_Page_Indexed_Indirect_by_X),
    0x28 => cpu.and(&Args::RegisterA, &Args::Immediate),
    0x29 => cpu.and(&Args::DirectPage, &Args::DirectPage),
    0x2A => cpu.or1(op),
    0x2B => cpu.rol(&Args::DirectPage),
    0x2C => cpu.rol(&Args::Absolute),
    0x2D => cpu.push(&Args::RegisterA),
    0x2E => cpu.cbne(&Args::DirectPage),
    0x2F => cpu.bra(),
    0x30 => cpu.bmi(),
    0x31 => cpu.tcall(op),
    0x32 => cpu.clr1(op, &Args::bit/* bit */),
    0x33 => cpu.bbc(op),
    0x34 => cpu.and(&Args::RegisterA, &Args::Direct_Page_Indexed_by_X),
    0x35 => cpu.and(&Args::RegisterA, &Args::Absolute_Indexed_by_X),
    0x36 => cpu.and(&Args::RegisterA, &Args::Absolute_Indexed_by_Y),
    0x37 => cpu.and(&Args::RegisterA, &Args::Direct_Page_Indirect_Indexed_by_Y),
    0x38 => cpu.and(&Args::DirectPage, &Args::Immediate),
    0x39 => cpu.and(&Args::RegisterX_Direct_Page_Indirect, &Args::CY/* (Y) */),
    0x3A => cpu.incw(&Args::DirectPage),
    0x3B => cpu.rol(&Args::Direct_Page_Indexed_by_X),
    0x3C => cpu.rol(&Args::RegisterA),
    0x3D => cpu.inc(&Args::RegisterX),
    0x3E => cpu.cmp(&Args::RegisterX, &Args::DirectPage),
    0x3F => cpu.call(&Args::Absolute),
    0x40 => cpu.setp(),
    0x41 => cpu.tcall(op),
    0x42 => cpu.set1(op, &Args::bit/* bit */),
    0x43 => cpu.bbs(op),
    0x44 => cpu.eor(&Args::RegisterA, &Args::DirectPage),
    0x45 => cpu.eor(&Args::RegisterA, &Args::Absolute),
    0x46 => cpu.eor(&Args::RegisterA, &Args::RegisterX_Direct_Page_Indirect),
    0x47 => cpu.eor(&Args::RegisterA, &Args::Direct_Page_Indexed_Indirect_by_X),
    0x48 => cpu.eor(&Args::RegisterA, &Args::Immediate),
    0x49 => cpu.eor(&Args::DirectPage, &Args::DirectPage),
    0x4A => cpu.and1(op),
    0x4B => cpu.lsr(&Args::DirectPage),
    0x4C => cpu.lsr(&Args::Absolute),
    0x4D => cpu.push(&Args::RegisterX),
    0x4E => cpu.tclr1(&Args::Absolute),
    0x4F => cpu.pcall(),
    0x50 => cpu.bvc(),
    0x51 => cpu.tcall(op),
    0x52 => cpu.clr1(op, &Args::bit/* bit */),
    0x53 => cpu.bbc(op),
    0x54 => cpu.eor(&Args::RegisterA, &Args::Direct_Page_Indexed_by_X),
    0x55 => cpu.eor(&Args::RegisterA, &Args::Absolute_Indexed_by_X),
    0x56 => cpu.eor(&Args::RegisterA, &Args::Absolute_Indexed_by_Y),
    0x57 => cpu.eor(&Args::RegisterA, &Args::Direct_Page_Indirect_Indexed_by_Y),
    0x58 => cpu.eor(&Args::DirectPage, &Args::Immediate),
    0x59 => cpu.eor(&Args::RegisterX_Direct_Page_Indirect, &Args::CY/* (Y) */),
    0x5A => cpu.cmpw(&Args::RegisterYA, &Args::DirectPage),
    0x5B => cpu.lsr(&Args::Direct_Page_Indexed_by_X),
    0x5C => cpu.lsr(&Args::RegisterA),
    0x5D => cpu.mov(&Args::RegisterX, &Args::RegisterA),
    0x5E => cpu.cmp(&Args::RegisterA, &Args::Immediate),
    0x5F => cpu.jmp(&Args::Absolute),
    0x60 => cpu.clrc(),
    0x61 => cpu.tcall(op),
    0x62 => cpu.set1(op, &Args::bit/* bit */),
    0x63 => cpu.bbs(op),
    0x64 => cpu.cmp(&Args::RegisterA, &Args::DirectPage),
    0x65 => cpu.cmp(&Args::RegisterA, &Args::Absolute),
    0x66 => cpu.cmp(&Args::RegisterA, &Args::RegisterX_Direct_Page_Indirect),
    0x67 => cpu.cmp(&Args::RegisterA, &Args::Direct_Page_Indexed_Indirect_by_X),
    0x68 => cpu.cmp(&Args::RegisterA, &Args::Immediate),
    0x69 => cpu.cmp(&Args::DirectPage, &Args::DirectPage),
    0x6A => cpu.and1(op),
    0x6B => cpu.ror(&Args::DirectPage),
    0x6C => cpu.ror(&Args::Absolute),
    0x6D => cpu.push(&Args::RegisterY),
    0x6E => cpu.dbnz(&Args::DirectPage),
    0x6F => cpu.ret(),
    0x70 => cpu.bvs(),
    0x71 => cpu.tcall(op),
    0x72 => cpu.clr1(op, &Args::bit/* bit */),
    0x73 => cpu.bbc(op),
    0x74 => cpu.cmp(&Args::RegisterA, &Args::Direct_Page_Indexed_by_X),
    0x75 => cpu.cmp(&Args::RegisterA, &Args::Absolute_Indexed_by_X),
    0x76 => cpu.cmp(&Args::RegisterA, &Args::Absolute_Indexed_by_Y),
    0x77 => cpu.cmp(&Args::RegisterA, &Args::Direct_Page_Indirect_Indexed_by_Y),
    0x78 => cpu.cmp(&Args::DirectPage, &Args::Immediate),
    0x79 => cpu.cmp(&Args::RegisterX_Direct_Page_Indirect, &Args::CY/* (Y) */),
    0x7A => cpu.addw(&Args::RegisterYA, &Args::DirectPage),
    0x7B => cpu.ror(&Args::Direct_Page_Indexed_by_X),
    0x7C => cpu.ror(&Args::RegisterA),
    0x7D => cpu.mov(&Args::RegisterA, &Args::RegisterX),
    0x7E => cpu.cmp(&Args::RegisterY, &Args::DirectPage),
    0x7F => cpu.reti(),
    0x80 => cpu.setc(),
    0x81 => cpu.tcall(op),
    0x82 => cpu.set1(op, &Args::bit/* bit */),
    0x83 => cpu.bbs(op),
    0x84 => cpu.adc(&Args::RegisterA, &Args::DirectPage),
    0x85 => cpu.adc(&Args::RegisterA, &Args::Absolute),
    0x86 => cpu.adc(&Args::RegisterA, &Args::RegisterX_Direct_Page_Indirect),
    0x87 => cpu.adc(&Args::RegisterA, &Args::Direct_Page_Indexed_Indirect_by_X),
    0x88 => cpu.adc(&Args::RegisterA, &Args::Immediate),
    0x89 => cpu.adc(&Args::DirectPage, &Args::DirectPage),
    0x8A => cpu.eor1(),
    0x8B => cpu.dec(&Args::DirectPage),
    0x8C => cpu.dec(&Args::Absolute),
    0x8D => cpu.mov(&Args::RegisterY, &Args::Immediate),
    0x8E => cpu.pop(&Args::ProgramStatus),
    0x8F => cpu.mov(&Args::DirectPage, &Args::Immediate),
    0x90 => cpu.bcc(),
    0x91 => cpu.tcall(op),
    0x92 => cpu.clr1(op, &Args::bit/* bit */),
    0x93 => cpu.bbc(op),
    0x94 => cpu.adc(&Args::RegisterA, &Args::Direct_Page_Indexed_by_X),
    0x95 => cpu.adc(&Args::RegisterA, &Args::Absolute_Indexed_by_X),
    0x96 => cpu.adc(&Args::RegisterA, &Args::Absolute_Indexed_by_Y),
    0x97 => cpu.adc(&Args::RegisterA, &Args::Direct_Page_Indirect_Indexed_by_Y),
    0x98 => cpu.adc(&Args::DirectPage, &Args::Immediate),
    0x99 => cpu.adc(&Args::RegisterX_Direct_Page_Indirect, &Args::CY/* (Y) */),
    0x9A => cpu.subw(&Args::RegisterYA, &Args::DirectPage),
    0x9B => cpu.dec(&Args::Direct_Page_Indexed_by_X),
    0x9C => cpu.dec(&Args::RegisterA),
    0x9D => cpu.mov(&Args::RegisterX, &Args::StackPointer),
    0x9E => cpu.div(),
    0x9F => cpu.xcn(),
    0xA0 => cpu.ei(),
    0xA1 => cpu.tcall(op),
    0xA2 => cpu.set1(op, &Args::bit/* bit */),
    0xA3 => cpu.bbs(op),
    0xA4 => cpu.sbc(&Args::RegisterA, &Args::DirectPage),
    0xA5 => cpu.sbc(&Args::RegisterA, &Args::Absolute),
    0xA6 => cpu.sbc(&Args::RegisterA, &Args::RegisterX_Direct_Page_Indirect),
    0xA7 => cpu.sbc(&Args::RegisterA, &Args::Direct_Page_Indexed_Indirect_by_X),
    0xA8 => cpu.sbc(&Args::RegisterA, &Args::Immediate),
    0xA9 => cpu.sbc(&Args::DirectPage, &Args::DirectPage),
    0xAA => cpu.mov1(&Args::C/* C */, &Args::mem/* mem. */, &Args::bit/* bit */),
    0xAB => cpu.inc(&Args::DirectPage),
    0xAC => cpu.inc(&Args::Absolute),
    0xAD => cpu.cmp(&Args::RegisterY, &Args::DirectPage),
    0xAE => cpu.pop(&Args::RegisterA),
    0xAF => cpu.mov(&Args::RegisterX_Direct_Page_Indirect_Incriment, &Args::RegisterA),
    0xB0 => cpu.bcs(),
    0xB1 => cpu.tcall(op),
    0xB2 => cpu.clr1(op, &Args::bit/* bit */),
    0xB3 => cpu.bbc(op),
    0xB4 => cpu.sbc(&Args::RegisterA, &Args::Direct_Page_Indexed_by_X),
    0xB5 => cpu.sbc(&Args::RegisterA, &Args::Absolute_Indexed_by_X),
    0xB6 => cpu.sbc(&Args::RegisterA, &Args::Absolute_Indexed_by_Y),
    0xB7 => cpu.sbc(&Args::RegisterA, &Args::Direct_Page_Indirect_Indexed_by_Y),
    0xB8 => cpu.sbc(&Args::DirectPage, &Args::Immediate),
    0xB9 => cpu.sbc(&Args::RegisterX_Direct_Page_Indirect, &Args::CY/* (Y) */),
    0xBA => cpu.movw(&Args::RegisterYA, &Args::DirectPage),
    0xBB => cpu.inc(&Args::Direct_Page_Indexed_by_X),
    0xBC => cpu.inc(&Args::RegisterA),
    0xBD => cpu.mov(&Args::StackPointer, &Args::RegisterX),
    0xBE => cpu.das(),
    0xBF => cpu.mov(&Args::RegisterA, &Args::RegisterX_Direct_Page_Indirect_Incriment),
    0xC0 => cpu.di(),
    0xC1 => cpu.tcall(op),
    0xC2 => cpu.set1(op, &Args::bit/* bit */),
    0xC3 => cpu.bbs(op),
    0xC4 => cpu.mov(&Args::DirectPage, &Args::RegisterA),
    0xC5 => cpu.mov(&Args::Absolute, &Args::RegisterA),
    0xC6 => cpu.mov(&Args::RegisterX_Direct_Page_Indirect, &Args::RegisterA),
    0xC7 => cpu.mov(&Args::Direct_Page_Indexed_Indirect_by_X, &Args::RegisterA),
    0xC8 => cpu.cmp(&Args::RegisterX, &Args::Immediate),
    0xC9 => cpu.mov(&Args::Absolute, &Args::RegisterX),
    // 0xCA => cpu.mov1(op, &Args::bit/* bit */, &Args::C/* C */),
    0xCB => cpu.mov(&Args::DirectPage, &Args::RegisterY),
    0xCC => cpu.mov(&Args::Absolute, &Args::RegisterY),
    0xCD => cpu.mov(&Args::RegisterX, &Args::Immediate),
    0xCE => cpu.pop(&Args::RegisterX),
    0xCF => cpu.mul(),
    0xD0 => cpu.bne(),
    0xD1 => cpu.tcall(op),
    0xD2 => cpu.clr1(op, &Args::bit/* bit */),
    0xD3 => cpu.bbc(op),
    0xD4 => cpu.mov(&Args::Direct_Page_Indexed_by_X, &Args::RegisterA),
    0xD5 => cpu.mov(&Args::Absolute_Indexed_by_X, &Args::RegisterA),
    0xD6 => cpu.mov(&Args::Absolute_Indexed_by_Y, &Args::RegisterA),
    0xD7 => cpu.mov(&Args::Direct_Page_Indirect_Indexed_by_Y, &Args::RegisterA),
    0xD8 => cpu.mov(&Args::DirectPage, &Args::RegisterX),
    0xD9 => cpu.mov(&Args::Direct_Page_Indexed_by_Y, &Args::RegisterX),
    0xDA => cpu.movw(&Args::DirectPage, &Args::RegisterYA),
    0xDB => cpu.mov(&Args::Direct_Page_Indexed_by_X, &Args::RegisterY),
    0xDC => cpu.dec(&Args::RegisterY),
    0xDD => cpu.mov(&Args::RegisterA, &Args::RegisterY),
    0xDE => cpu.cbne(&Args::Direct_Page_Indexed_by_X),
    0xDF => cpu.daa(),
    0xE0 => cpu.clrv(),
    0xE1 => cpu.tcall(op),
    0xE2 => cpu.set1(op, &Args::bit/* bit */),
    0xE3 => cpu.bbs(op),
    0xE4 => cpu.mov(&Args::RegisterA, &Args::DirectPage),
    0xE5 => cpu.mov(&Args::RegisterA, &Args::Absolute),
    0xE6 => cpu.mov(&Args::RegisterA, &Args::RegisterX_Direct_Page_Indirect),
    0xE7 => cpu.mov(&Args::RegisterA, &Args::Direct_Page_Indexed_Indirect_by_X),
    0xE8 => cpu.mov(&Args::RegisterA, &Args::Immediate),
    0xE9 => cpu.mov(&Args::RegisterX, &Args::Absolute),
    0xEA => cpu.not1(op, &Args::bit/* bit */),
    0xEB => cpu.mov(&Args::RegisterY, &Args::DirectPage),
    0xEC => cpu.mov(&Args::RegisterY, &Args::Absolute),
    0xED => cpu.notc(),
    0xEE => cpu.pop(&Args::RegisterY),
    0xEF => cpu.sleep(),
    0xF0 => cpu.beq(),
    0xF1 => cpu.tcall(op),
    0xF2 => cpu.clr1(op, &Args::bit/* bit */),
    0xF3 => cpu.bbc(op),
    0xF4 => cpu.mov(&Args::RegisterA, &Args::Direct_Page_Indexed_by_X),
    0xF5 => cpu.mov(&Args::RegisterA, &Args::Absolute_Indexed_by_X),
    0xF6 => cpu.mov(&Args::RegisterA, &Args::Absolute_Indexed_by_Y),
    0xF7 => cpu.mov(&Args::RegisterA, &Args::Direct_Page_Indirect_Indexed_by_Y),
    0xF8 => cpu.mov(&Args::RegisterX, &Args::DirectPage),
    0xF9 => cpu.mov(&Args::RegisterX, &Args::Direct_Page_Indexed_by_Y),
    0xFA => cpu.mov(&Args::DirectPage, &Args::DirectPage),
    0xFB => cpu.mov(&Args::RegisterY, &Args::Direct_Page_Indexed_by_X),
    0xFC => cpu.inc(&Args::RegisterY),
    0xFD => cpu.mov(&Args::RegisterY, &Args::RegisterA),
    0xFE => cpu.dbnz(&Args::RegisterY),
    0xFF => cpu.stop(),
    _ => panic!("")
  }
}
