
fn cycles(op: u8) -> u8 {
  match op
    0x00 => 2, // cycles
    ...
    0xFF => 3,
  }
}


fn run_op(op: u8) {
  match op {
    0x00 => self.nop(),
    0xCD => self.mov(&Args::RegisterX, &Args::Immediate),
    ...
  }
}

// MOV A, #imm :: Immediate	E8	2 bytes	3 cycles
