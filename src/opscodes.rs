use std::{collections::HashMap};
use once_cell::sync::Lazy;
use crate::cpu::{AddressingMode, CPU, FLAG_MEMORY_ACCUMULATOR_MODE, MODE_16BIT, OpCode, OpInfo};

pub static CPU_OPS_CODES: Lazy<HashMap<u8, OpCode>> = Lazy::new(|| {
  let mut m = HashMap::new();
  m.insert(0xD0, OpCode::new(0xD0, "BNE", OpInfo::new(2, 2), OpInfo::new(2, 2), AddressingMode::Program_Counter_Relative));
  m.insert(0x8A, OpCode::new(0x8A, "TXA", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0xAA, OpCode::new(0xAA, "TAX", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x8C, OpCode::new(0x8C, "STY", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0x84, OpCode::new(0x84, "STY", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0x94, OpCode::new(0x94, "STY", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x50, OpCode::new(0x50, "BVC", OpInfo::new(2, 2), OpInfo::new(2, 2), AddressingMode::Program_Counter_Relative));
  m.insert(0x8B, OpCode::new(0x8B, "PHB", OpInfo::new(1, 3), OpInfo::new(1, 3), AddressingMode::Stack));
  m.insert(0x62, OpCode::new(0x62, "PER", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Stack));
  m.insert(0xAB, OpCode::new(0xAB, "PLB", OpInfo::new(1, 4), OpInfo::new(1, 4), AddressingMode::Stack));
  m.insert(0x40, OpCode::new(0x40, "RTI", OpInfo::new(1, 7), OpInfo::new(1, 7), AddressingMode::Stack));
  m.insert(0x38, OpCode::new(0x38, "SEC", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x1C, OpCode::new(0x1C, "TRB", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute));
  m.insert(0x14, OpCode::new(0x14, "TRB", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page));
  m.insert(0x69, OpCode::new(0x69, "ADC", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0x6D, OpCode::new(0x6D, "ADC", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0x6F, OpCode::new(0x6F, "ADC", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long));
  m.insert(0x65, OpCode::new(0x65, "ADC", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0x72, OpCode::new(0x72, "ADC", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect));
  m.insert(0x67, OpCode::new(0x67, "ADC", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long));
  m.insert(0x7D, OpCode::new(0x7D, "ADC", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x7F, OpCode::new(0x7F, "ADC", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long_Indexed_by_X));
  m.insert(0x79, OpCode::new(0x79, "ADC", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_Y));
  m.insert(0x75, OpCode::new(0x75, "ADC", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x61, OpCode::new(0x61, "ADC", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_Indirect_by_X));
  m.insert(0x71, OpCode::new(0x71, "ADC", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect_Indexed_by_Y));
  m.insert(0x77, OpCode::new(0x77, "ADC", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long_Indexed_by_Y));
  m.insert(0x63, OpCode::new(0x63, "ADC", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Stack_Relative));
  m.insert(0x73, OpCode::new(0x73, "ADC", OpInfo::new(2, 7), OpInfo::new(2, 7), AddressingMode::Stack_Relative_Indirect_Indexed_by_Y));
  m.insert(0x8E, OpCode::new(0x8E, "STX", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0x86, OpCode::new(0x86, "STX", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0x96, OpCode::new(0x96, "STX", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_Y));
  m.insert(0x9B, OpCode::new(0x9B, "TXY", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x4C, OpCode::new(0x4C, "JMP", OpInfo::new(3, 3), OpInfo::new(3, 3), AddressingMode::Absolute));
  m.insert(0x6C, OpCode::new(0x6C, "JMP", OpInfo::new(3, 5), OpInfo::new(3, 5), AddressingMode::Absolute_Indirect));
  m.insert(0x7C, OpCode::new(0x7C, "JMP", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute_Indexed_Indirect));
  m.insert(0x5C, OpCode::new(0x5C, "JMP", OpInfo::new(4, 4), OpInfo::new(4, 4), AddressingMode::Absolute_Long));
  m.insert(0xDC, OpCode::new(0xDC, "JMP", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute_Indirect_Long));
  m.insert(0xCB, OpCode::new(0xCB, "WAI", OpInfo::new(1, 3), OpInfo::new(1, 3), AddressingMode::Implied));
  m.insert(0x8D, OpCode::new(0x8D, "STA", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0x8F, OpCode::new(0x8F, "STA", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long));
  m.insert(0x85, OpCode::new(0x85, "STA", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0x92, OpCode::new(0x92, "STA", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect));
  m.insert(0x87, OpCode::new(0x87, "STA", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long));
  m.insert(0x9D, OpCode::new(0x9D, "STA", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x9F, OpCode::new(0x9F, "STA", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long_Indexed_by_X));
  m.insert(0x99, OpCode::new(0x99, "STA", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_Y));
  m.insert(0x95, OpCode::new(0x95, "STA", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x81, OpCode::new(0x81, "STA", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_Indirect_by_X));
  m.insert(0x91, OpCode::new(0x91, "STA", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect_Indexed_by_Y));
  m.insert(0x97, OpCode::new(0x97, "STA", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long_Indexed_by_Y));
  m.insert(0x83, OpCode::new(0x83, "STA", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Stack_Relative));
  m.insert(0x93, OpCode::new(0x93, "STA", OpInfo::new(2, 7), OpInfo::new(2, 7), AddressingMode::Stack_Relative_Indirect_Indexed_by_Y));
  m.insert(0xA8, OpCode::new(0xA8, "TAY", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x0A, OpCode::new(0x0A, "ASL", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Accumulator));
  m.insert(0x0E, OpCode::new(0x0E, "ASL", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute));
  m.insert(0x06, OpCode::new(0x06, "ASL", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page));
  m.insert(0x1E, OpCode::new(0x1E, "ASL", OpInfo::new(3, 7), OpInfo::new(3, 7), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x16, OpCode::new(0x16, "ASL", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0xF4, OpCode::new(0xF4, "PEA", OpInfo::new(3, 5), OpInfo::new(3, 5), AddressingMode::Stack));
  m.insert(0xBB, OpCode::new(0xBB, "TYX", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x28, OpCode::new(0x28, "PLP", OpInfo::new(1, 4), OpInfo::new(1, 4), AddressingMode::Stack));
  m.insert(0xE8, OpCode::new(0xE8, "INX", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0xC0, OpCode::new(0xC0, "CPY", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0xCC, OpCode::new(0xCC, "CPY", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0xC4, OpCode::new(0xC4, "CPY", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0x3B, OpCode::new(0x3B, "TSC", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x0B, OpCode::new(0x0B, "PHD", OpInfo::new(1, 4), OpInfo::new(1, 4), AddressingMode::Stack));
  m.insert(0xF8, OpCode::new(0xF8, "SED", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0xE9, OpCode::new(0xE9, "SBC", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0xED, OpCode::new(0xED, "SBC", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0xEF, OpCode::new(0xEF, "SBC", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long));
  m.insert(0xE5, OpCode::new(0xE5, "SBC", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0xF2, OpCode::new(0xF2, "SBC", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect));
  m.insert(0xE7, OpCode::new(0xE7, "SBC", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long));
  m.insert(0xFD, OpCode::new(0xFD, "SBC", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0xFF, OpCode::new(0xFF, "SBC", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long_Indexed_by_X));
  m.insert(0xF9, OpCode::new(0xF9, "SBC", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_Y));
  m.insert(0xF5, OpCode::new(0xF5, "SBC", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0xE1, OpCode::new(0xE1, "SBC", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_Indirect_by_X));
  m.insert(0xF1, OpCode::new(0xF1, "SBC", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect_Indexed_by_Y));
  m.insert(0xF7, OpCode::new(0xF7, "SBC", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long_Indexed_by_Y));
  m.insert(0xE3, OpCode::new(0xE3, "SBC", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Stack_Relative));
  m.insert(0xF3, OpCode::new(0xF3, "SBC", OpInfo::new(2, 7), OpInfo::new(2, 7), AddressingMode::Stack_Relative_Indirect_Indexed_by_Y));
  m.insert(0xB0, OpCode::new(0xB0, "BCS", OpInfo::new(2, 2), OpInfo::new(2, 2), AddressingMode::Program_Counter_Relative));
  m.insert(0xA0, OpCode::new(0xA0, "LDY", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0xAC, OpCode::new(0xAC, "LDY", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0xA4, OpCode::new(0xA4, "LDY", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0xBC, OpCode::new(0xBC, "LDY", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0xB4, OpCode::new(0xB4, "LDY", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0xCA, OpCode::new(0xCA, "DEX", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x49, OpCode::new(0x49, "EOR", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0x4D, OpCode::new(0x4D, "EOR", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0x4F, OpCode::new(0x4F, "EOR", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long));
  m.insert(0x45, OpCode::new(0x45, "EOR", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0x52, OpCode::new(0x52, "EOR", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect));
  m.insert(0x47, OpCode::new(0x47, "EOR", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long));
  m.insert(0x5D, OpCode::new(0x5D, "EOR", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x5F, OpCode::new(0x5F, "EOR", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long_Indexed_by_X));
  m.insert(0x59, OpCode::new(0x59, "EOR", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_Y));
  m.insert(0x55, OpCode::new(0x55, "EOR", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x41, OpCode::new(0x41, "EOR", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_Indirect_by_X));
  m.insert(0x51, OpCode::new(0x51, "EOR", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect_Indexed_by_Y));
  m.insert(0x57, OpCode::new(0x57, "EOR", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long_Indexed_by_Y));
  m.insert(0x43, OpCode::new(0x43, "EOR", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Stack_Relative));
  m.insert(0x53, OpCode::new(0x53, "EOR", OpInfo::new(2, 7), OpInfo::new(2, 7), AddressingMode::Stack_Relative_Indirect_Indexed_by_Y));
  m.insert(0x30, OpCode::new(0x30, "BMI", OpInfo::new(2, 2), OpInfo::new(2, 2), AddressingMode::Program_Counter_Relative));
  m.insert(0xEB, OpCode::new(0xEB, "XBA", OpInfo::new(1, 3), OpInfo::new(1, 3), AddressingMode::Implied));
  m.insert(0xEA, OpCode::new(0xEA, "NOP", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x98, OpCode::new(0x98, "TYA", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x78, OpCode::new(0x78, "SEI", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x88, OpCode::new(0x88, "DEY", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x1B, OpCode::new(0x1B, "TCS", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0xA2, OpCode::new(0xA2, "LDX", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0xAE, OpCode::new(0xAE, "LDX", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0xA6, OpCode::new(0xA6, "LDX", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0xBE, OpCode::new(0xBE, "LDX", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_Y));
  m.insert(0xB6, OpCode::new(0xB6, "LDX", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_Y));
  m.insert(0x9A, OpCode::new(0x9A, "TXS", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x2B, OpCode::new(0x2B, "PLD", OpInfo::new(1, 5), OpInfo::new(1, 5), AddressingMode::Stack));
  m.insert(0x09, OpCode::new(0x09, "ORA", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0x0D, OpCode::new(0x0D, "ORA", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0x0F, OpCode::new(0x0F, "ORA", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long));
  m.insert(0x05, OpCode::new(0x05, "ORA", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0x12, OpCode::new(0x12, "ORA", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect));
  m.insert(0x07, OpCode::new(0x07, "ORA", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long));
  m.insert(0x1D, OpCode::new(0x1D, "ORA", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x1F, OpCode::new(0x1F, "ORA", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long_Indexed_by_X));
  m.insert(0x19, OpCode::new(0x19, "ORA", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_Y));
  m.insert(0x15, OpCode::new(0x15, "ORA", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x01, OpCode::new(0x01, "ORA", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_Indirect_by_X));
  m.insert(0x11, OpCode::new(0x11, "ORA", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect_Indexed_by_Y));
  m.insert(0x17, OpCode::new(0x17, "ORA", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long_Indexed_by_Y));
  m.insert(0x03, OpCode::new(0x03, "ORA", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Stack_Relative));
  m.insert(0x13, OpCode::new(0x13, "ORA", OpInfo::new(2, 7), OpInfo::new(2, 7), AddressingMode::Stack_Relative_Indirect_Indexed_by_Y));
  m.insert(0x0C, OpCode::new(0x0C, "TSB", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute));
  m.insert(0x04, OpCode::new(0x04, "TSB", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page));
  m.insert(0xE0, OpCode::new(0xE0, "CPX", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0xEC, OpCode::new(0xEC, "CPX", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0xE4, OpCode::new(0xE4, "CPX", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0xA9, OpCode::new(0xA9, "LDA", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0xAD, OpCode::new(0xAD, "LDA", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0xAF, OpCode::new(0xAF, "LDA", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long));
  m.insert(0xA5, OpCode::new(0xA5, "LDA", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0xB2, OpCode::new(0xB2, "LDA", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect));
  m.insert(0xA7, OpCode::new(0xA7, "LDA", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long));
  m.insert(0xBD, OpCode::new(0xBD, "LDA", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0xBF, OpCode::new(0xBF, "LDA", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long_Indexed_by_X));
  m.insert(0xB9, OpCode::new(0xB9, "LDA", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_Y));
  m.insert(0xB5, OpCode::new(0xB5, "LDA", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0xA1, OpCode::new(0xA1, "LDA", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_Indirect_by_X));
  m.insert(0xB1, OpCode::new(0xB1, "LDA", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect_Indexed_by_Y));
  m.insert(0xB7, OpCode::new(0xB7, "LDA", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long_Indexed_by_Y));
  m.insert(0xA3, OpCode::new(0xA3, "LDA", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Stack_Relative));
  m.insert(0xB3, OpCode::new(0xB3, "LDA", OpInfo::new(2, 7), OpInfo::new(2, 7), AddressingMode::Stack_Relative_Indirect_Indexed_by_Y));
  m.insert(0x08, OpCode::new(0x08, "PHP", OpInfo::new(1, 3), OpInfo::new(1, 3), AddressingMode::Stack));
  m.insert(0xC8, OpCode::new(0xC8, "INY", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0xE2, OpCode::new(0xE2, "SEP", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Immediate));
  m.insert(0x6B, OpCode::new(0x6B, "RTL", OpInfo::new(1, 6), OpInfo::new(1, 6), AddressingMode::Stack));
  m.insert(0x2A, OpCode::new(0x2A, "ROL", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Accumulator));
  m.insert(0x2E, OpCode::new(0x2E, "ROL", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute));
  m.insert(0x26, OpCode::new(0x26, "ROL", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page));
  m.insert(0x3E, OpCode::new(0x3E, "ROL", OpInfo::new(3, 7), OpInfo::new(3, 7), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x36, OpCode::new(0x36, "ROL", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x3A, OpCode::new(0x3A, "DEC", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Accumulator));
  m.insert(0xCE, OpCode::new(0xCE, "DEC", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute));
  m.insert(0xC6, OpCode::new(0xC6, "DEC", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page));
  m.insert(0xDE, OpCode::new(0xDE, "DEC", OpInfo::new(3, 7), OpInfo::new(3, 7), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0xD6, OpCode::new(0xD6, "DEC", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x20, OpCode::new(0x20, "JSR", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute));
  m.insert(0xFC, OpCode::new(0xFC, "JSR", OpInfo::new(3, 8), OpInfo::new(3, 8), AddressingMode::Absolute_Indexed_Indirect));
  m.insert(0x22, OpCode::new(0x22, "JSR", OpInfo::new(4, 8), OpInfo::new(4, 8), AddressingMode::Absolute_Long));
  m.insert(0x29, OpCode::new(0x29, "AND", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0x2D, OpCode::new(0x2D, "AND", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0x2F, OpCode::new(0x2F, "AND", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long));
  m.insert(0x25, OpCode::new(0x25, "AND", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0x32, OpCode::new(0x32, "AND", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect));
  m.insert(0x27, OpCode::new(0x27, "AND", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long));
  m.insert(0x3D, OpCode::new(0x3D, "AND", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x3F, OpCode::new(0x3F, "AND", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long_Indexed_by_X));
  m.insert(0x39, OpCode::new(0x39, "AND", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_Y));
  m.insert(0x35, OpCode::new(0x35, "AND", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x21, OpCode::new(0x21, "AND", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_Indirect_by_X));
  m.insert(0x31, OpCode::new(0x31, "AND", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect_Indexed_by_Y));
  m.insert(0x37, OpCode::new(0x37, "AND", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long_Indexed_by_Y));
  m.insert(0x23, OpCode::new(0x23, "AND", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Stack_Relative));
  m.insert(0x33, OpCode::new(0x33, "AND", OpInfo::new(2, 7), OpInfo::new(2, 7), AddressingMode::Stack_Relative_Indirect_Indexed_by_Y));
  m.insert(0xFB, OpCode::new(0xFB, "XCE", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x1A, OpCode::new(0x1A, "INC", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Accumulator));
  m.insert(0xEE, OpCode::new(0xEE, "INC", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute));
  m.insert(0xE6, OpCode::new(0xE6, "INC", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page));
  m.insert(0xFE, OpCode::new(0xFE, "INC", OpInfo::new(3, 7), OpInfo::new(3, 7), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0xF6, OpCode::new(0xF6, "INC", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x58, OpCode::new(0x58, "CLI", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0xBA, OpCode::new(0xBA, "TSX", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x54, OpCode::new(0x54, "MVN", OpInfo::new(3, 7), OpInfo::new(3, 7), AddressingMode::Block_Move));
  m.insert(0xDB, OpCode::new(0xDB, "STP", OpInfo::new(1, 3), OpInfo::new(1, 3), AddressingMode::Implied));
  m.insert(0x5B, OpCode::new(0x5B, "TCD", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x4B, OpCode::new(0x4B, "PHK", OpInfo::new(1, 3), OpInfo::new(1, 3), AddressingMode::Stack));
  m.insert(0x00, OpCode::new(0x00, "BRK", OpInfo::new(2, 8), OpInfo::new(2, 8), AddressingMode::Stack));
  m.insert(0x7B, OpCode::new(0x7B, "TDC", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x70, OpCode::new(0x70, "BVS", OpInfo::new(2, 2), OpInfo::new(2, 2), AddressingMode::Program_Counter_Relative));
  m.insert(0xD8, OpCode::new(0xD8, "CLD", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0xF0, OpCode::new(0xF0, "BEQ", OpInfo::new(2, 2), OpInfo::new(2, 2), AddressingMode::Program_Counter_Relative));
  m.insert(0x82, OpCode::new(0x82, "BRL", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Program_Counter_Relative_Long));
  m.insert(0x02, OpCode::new(0x02, "COP", OpInfo::new(2, 8), OpInfo::new(2, 8), AddressingMode::Stack));
  m.insert(0x42, OpCode::new(0x42, "WDM", OpInfo::new(2, 2), OpInfo::new(2, 2), AddressingMode::Implied));
  m.insert(0x5A, OpCode::new(0x5A, "PHY", OpInfo::new(1, 3), OpInfo::new(1, 3), AddressingMode::Stack));
  m.insert(0xFA, OpCode::new(0xFA, "PLX", OpInfo::new(1, 4), OpInfo::new(1, 4), AddressingMode::Stack));
  m.insert(0x60, OpCode::new(0x60, "RTS", OpInfo::new(1, 6), OpInfo::new(1, 6), AddressingMode::Stack));
  m.insert(0xD4, OpCode::new(0xD4, "PEI", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Stack));
  m.insert(0x68, OpCode::new(0x68, "PLA", OpInfo::new(1, 4), OpInfo::new(1, 4), AddressingMode::Stack));
  m.insert(0x4A, OpCode::new(0x4A, "LSR", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Accumulator));
  m.insert(0x4E, OpCode::new(0x4E, "LSR", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute));
  m.insert(0x46, OpCode::new(0x46, "LSR", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page));
  m.insert(0x5E, OpCode::new(0x5E, "LSR", OpInfo::new(3, 7), OpInfo::new(3, 7), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x56, OpCode::new(0x56, "LSR", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x18, OpCode::new(0x18, "CLC", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0xB8, OpCode::new(0xB8, "CLV", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Implied));
  m.insert(0x10, OpCode::new(0x10, "BPL", OpInfo::new(2, 2), OpInfo::new(2, 2), AddressingMode::Program_Counter_Relative));
  m.insert(0xC9, OpCode::new(0xC9, "CMP", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0xCD, OpCode::new(0xCD, "CMP", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0xCF, OpCode::new(0xCF, "CMP", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long));
  m.insert(0xC5, OpCode::new(0xC5, "CMP", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0xD2, OpCode::new(0xD2, "CMP", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect));
  m.insert(0xC7, OpCode::new(0xC7, "CMP", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long));
  m.insert(0xDD, OpCode::new(0xDD, "CMP", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0xDF, OpCode::new(0xDF, "CMP", OpInfo::new(4, 5), OpInfo::new(4, 5), AddressingMode::Absolute_Long_Indexed_by_X));
  m.insert(0xD9, OpCode::new(0xD9, "CMP", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_Y));
  m.insert(0xD5, OpCode::new(0xD5, "CMP", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0xC1, OpCode::new(0xC1, "CMP", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_Indirect_by_X));
  m.insert(0xD1, OpCode::new(0xD1, "CMP", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page_Indirect_Indexed_by_Y));
  m.insert(0xD7, OpCode::new(0xD7, "CMP", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indirect_Long_Indexed_by_Y));
  m.insert(0xC3, OpCode::new(0xC3, "CMP", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Stack_Relative));
  m.insert(0xD3, OpCode::new(0xD3, "CMP", OpInfo::new(2, 7), OpInfo::new(2, 7), AddressingMode::Stack_Relative_Indirect_Indexed_by_Y));
  m.insert(0x89, OpCode::new(0x89, "BIT", OpInfo::new(3, 2), OpInfo::new(2, 2), AddressingMode::Immediate));
  m.insert(0x2C, OpCode::new(0x2C, "BIT", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0x24, OpCode::new(0x24, "BIT", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0x3C, OpCode::new(0x3C, "BIT", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x34, OpCode::new(0x34, "BIT", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0xC2, OpCode::new(0xC2, "REP", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Immediate));
  m.insert(0x9C, OpCode::new(0x9C, "STZ", OpInfo::new(3, 4), OpInfo::new(3, 4), AddressingMode::Absolute));
  m.insert(0x64, OpCode::new(0x64, "STZ", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Direct_Page));
  m.insert(0x9E, OpCode::new(0x9E, "STZ", OpInfo::new(3, 5), OpInfo::new(3, 5), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x74, OpCode::new(0x74, "STZ", OpInfo::new(2, 4), OpInfo::new(2, 4), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x48, OpCode::new(0x48, "PHA", OpInfo::new(1, 3), OpInfo::new(1, 3), AddressingMode::Stack));
  m.insert(0x44, OpCode::new(0x44, "MVP", OpInfo::new(3, 7), OpInfo::new(3, 7), AddressingMode::Block_Move));
  m.insert(0x80, OpCode::new(0x80, "BRA", OpInfo::new(2, 3), OpInfo::new(2, 3), AddressingMode::Program_Counter_Relative));
  m.insert(0x6A, OpCode::new(0x6A, "ROR", OpInfo::new(1, 2), OpInfo::new(1, 2), AddressingMode::Accumulator));
  m.insert(0x6E, OpCode::new(0x6E, "ROR", OpInfo::new(3, 6), OpInfo::new(3, 6), AddressingMode::Absolute));
  m.insert(0x66, OpCode::new(0x66, "ROR", OpInfo::new(2, 5), OpInfo::new(2, 5), AddressingMode::Direct_Page));
  m.insert(0x7E, OpCode::new(0x7E, "ROR", OpInfo::new(3, 7), OpInfo::new(3, 7), AddressingMode::Absolute_Indexed_by_X));
  m.insert(0x76, OpCode::new(0x76, "ROR", OpInfo::new(2, 6), OpInfo::new(2, 6), AddressingMode::Direct_Page_Indexed_by_X));
  m.insert(0x7A, OpCode::new(0x7A, "PLY", OpInfo::new(1, 4), OpInfo::new(1, 4), AddressingMode::Stack));
  m.insert(0xDA, OpCode::new(0xDA, "PHX", OpInfo::new(1, 3), OpInfo::new(1, 3), AddressingMode::Stack));
  m.insert(0x90, OpCode::new(0x90, "BCC", OpInfo::new(2, 2), OpInfo::new(2, 2), AddressingMode::Program_Counter_Relative));
m
});
/*
{
  "Program Counter Relative": {
    "BNE": true,
    "BVC": true,
    "BCS": true,
    "BMI": true,
    "BVS": true,
    "BEQ": true,
    "BPL": true,
    "BRA": true,
    "BCC": true
  },
  "Implied (type 1)": {
    "TXA": true,
    "TAX": true,
    "TXY": true,
    "TAY": true,
    "TYX": true,
    "INX": true,
    "TSC": true,
    "DEX": true,
    "XBA": true,
    "TYA": true,
    "DEY": true,
    "TCS": true,
    "TXS": true,
    "INY": true,
    "TSX": true,
    "TCD": true,
    "TDC": true
  },
  "Absolute": {
    "STY": true,
    "TRB": true,
    "ADC": true,
    "STX": true,
    "JMP": true,
    "STA": true,
    "ASL": true,
    "CPY": true,
    "SBC": true,
    "LDY": true,
    "EOR": true,
    "LDX": true,
    "ORA": true,
    "TSB": true,
    "CPX": true,
    "LDA": true,
    "ROL": true,
    "DEC": true,
    "JSR": true,
    "AND": true,
    "INC": true,
    "LSR": true,
    "CMP": true,
    "BIT": true,
    "STZ": true,
    "ROR": true
  },
  "Direct Page": {
    "STY": true,
    "TRB": true,
    "ADC": true,
    "STX": true,
    "STA": true,
    "ASL": true,
    "CPY": true,
    "SBC": true,
    "LDY": true,
    "EOR": true,
    "LDX": true,
    "ORA": true,
    "TSB": true,
    "CPX": true,
    "LDA": true,
    "ROL": true,
    "DEC": true,
    "AND": true,
    "INC": true,
    "LSR": true,
    "CMP": true,
    "BIT": true,
    "STZ": true,
    "ROR": true
  },
  "Direct Page Indexed by X": {
    "STY": true,
    "ADC": true,
    "STA": true,
    "ASL": true,
    "SBC": true,
    "LDY": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "ROL": true,
    "DEC": true,
    "AND": true,
    "INC": true,
    "LSR": true,
    "CMP": true,
    "BIT": true,
    "STZ": true,
    "ROR": true
  },
  "Stack (Push)": {
    "PHB": true,
    "PHD": true,
    "PHP": true,
    "PHK": true,
    "PHY": true,
    "PHA": true,
    "PHX": true
  },
  "Stack (PC Relative Long)": {
    "PER": true
  },
  "Stack (Pull)": {
    "PLB": true,
    "PLP": true,
    "PLD": true,
    "PLX": true,
    "PLA": true,
    "PLY": true
  },
  "Stack (RTI)": {
    "RTI": true
  },
  "Implied (type 2)": {
    "SEC": true,
    "SED": true,
    "SEI": true,
    "XCE": true,
    "CLI": true,
    "CLD": true,
    "CLC": true,
    "CLV": true
  },
  "Immediate": {
    "ADC": true,
    "CPY": true,
    "SBC": true,
    "LDY": true,
    "EOR": true,
    "LDX": true,
    "ORA": true,
    "CPX": true,
    "LDA": true,
    "SEP": true,
    "AND": true,
    "CMP": true,
    "BIT": true,
    "REP": true
  },
  "Absolute Long": {
    "ADC": true,
    "JMP": true,
    "STA": true,
    "SBC": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "JSR": true,
    "AND": true,
    "CMP": true
  },
  "Direct Page Indirect": {
    "ADC": true,
    "STA": true,
    "SBC": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "AND": true,
    "CMP": true
  },
  "Direct Page Indirect Long": {
    "ADC": true,
    "STA": true,
    "SBC": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "AND": true,
    "CMP": true
  },
  "Absolute Indexed by X": {
    "ADC": true,
    "STA": true,
    "ASL": true,
    "SBC": true,
    "LDY": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "ROL": true,
    "DEC": true,
    "AND": true,
    "INC": true,
    "LSR": true,
    "CMP": true,
    "BIT": true,
    "STZ": true,
    "ROR": true
  },
  "Absolute Long Indexed by X": {
    "ADC": true,
    "STA": true,
    "SBC": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "AND": true,
    "CMP": true
  },
  "Absolute Indexed by Y": {
    "ADC": true,
    "STA": true,
    "SBC": true,
    "EOR": true,
    "LDX": true,
    "ORA": true,
    "LDA": true,
    "AND": true,
    "CMP": true
  },
  "Direct Page Indexed Indirect by X": {
    "ADC": true,
    "STA": true,
    "SBC": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "AND": true,
    "CMP": true
  },
  "Direct Page Indirect Indexed by Y": {
    "ADC": true,
    "STA": true,
    "SBC": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "AND": true,
    "CMP": true
  },
  "Direct Page Indirect Long Indexed by Y": {
    "ADC": true,
    "STA": true,
    "SBC": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "AND": true,
    "CMP": true
  },
  "Stack Relative": {
    "ADC": true,
    "STA": true,
    "SBC": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "AND": true,
    "CMP": true
  },
  "Stack Relative Indirect Indexed by Y": {
    "ADC": true,
    "STA": true,
    "SBC": true,
    "EOR": true,
    "ORA": true,
    "LDA": true,
    "AND": true,
    "CMP": true
  },
  "Direct Page Indexed by Y": {
    "STX": true,
    "LDX": true
  },
  "Absolute Indirect": {
    "JMP": true
  },
  "Absolute Indexed Indirect": {
    "JMP": true,
    "JSR": true
  },
  "Absolute Indirect Long": {
    "JMP": true
  },
  "Implied (type 3)": {
    "WAI": true,
    "NOP": true,
    "STP": true
  },
  "Accumulator": {
    "ASL": true,
    "ROL": true,
    "DEC": true,
    "INC": true,
    "LSR": true,
    "ROR": true
  },
  "Stack (absolute)": {
    "PEA": true
  },
  "Stack (RTL)": {
    "RTL": true
  },
  "Block Move": {
    "MVN": true,
    "MVP": true
  },
  "Stack (Interrupt)": {
    "BRK": true,
    "COP": true
  },
  "Program Counter Relative Long": {
    "BRL": true
  },
  "Implied (type 3)[4]": {
    "WDM": true
  },
  "Stack (RTS)": {
    "RTS": true
  },
  "Stack (Direct Page Indirect)": {
    "PEI": true
  }
}
*/


pub fn call(cpu: &mut CPU, op: &OpCode) {
  match op.name.as_str() {
    

  "BNE" => {
      cpu.bne(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TXA" => {
      cpu.txa(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TAX" => {
      cpu.tax(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "STY" => {
      cpu.sty(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BVC" => {
      cpu.bvc(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PHB" => {
      cpu.phb(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PER" => {
      cpu.per(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PLB" => {
      cpu.plb(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "RTI" => {
      cpu.rti(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "SEC" => {
      cpu.sec(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TRB" => {
      cpu.trb(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "ADC" => {
      cpu.adc(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "STX" => {
      cpu.stx(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TXY" => {
      cpu.txy(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "JMP" => {
      cpu.jmp(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "WAI" => {
      cpu.wai(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "STA" => {
      cpu.sta(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TAY" => {
      cpu.tay(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "ASL" => {
      cpu.asl(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PEA" => {
      cpu.pea(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TYX" => {
      cpu.tyx(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PLP" => {
      cpu.plp(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "INX" => {
      cpu.inx(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "CPY" => {
      cpu.cpy(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TSC" => {
      cpu.tsc(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PHD" => {
      cpu.phd(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "SED" => {
      cpu.sed(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "SBC" => {
      cpu.sbc(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BCS" => {
      cpu.bcs(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "LDY" => {
      cpu.ldy(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "DEX" => {
      cpu.dex(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "EOR" => {
      cpu.eor(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BMI" => {
      cpu.bmi(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "XBA" => {
      cpu.xba(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "NOP" => {
      cpu.nop(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TYA" => {
      cpu.tya(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "SEI" => {
      cpu.sei(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "DEY" => {
      cpu.dey(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TCS" => {
      cpu.tcs(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "LDX" => {
      cpu.ldx(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TXS" => {
      cpu.txs(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PLD" => {
      cpu.pld(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "ORA" => {
      cpu.ora(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TSB" => {
      cpu.tsb(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "CPX" => {
      cpu.cpx(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "LDA" => {
      cpu.lda(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PHP" => {
      cpu.php(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "INY" => {
      cpu.iny(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "SEP" => {
      cpu.sep(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "RTL" => {
      cpu.rtl(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "ROL" => {
      cpu.rol(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "DEC" => {
      cpu.dec(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "JSR" => {
      cpu.jsr(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "AND" => {
      cpu.and(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "XCE" => {
      cpu.xce(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "INC" => {
      cpu.inc(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "CLI" => {
      cpu.cli(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TSX" => {
      cpu.tsx(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "MVN" => {
      cpu.mvn(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "STP" => {
      cpu.stp(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TCD" => {
      cpu.tcd(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PHK" => {
      cpu.phk(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BRK" => {
      cpu.brk(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "TDC" => {
      cpu.tdc(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BVS" => {
      cpu.bvs(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "CLD" => {
      cpu.cld(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BEQ" => {
      cpu.beq(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BRL" => {
      cpu.brl(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "COP" => {
      cpu.cop(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "WDM" => {
      cpu.wdm(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PHY" => {
      cpu.phy(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PLX" => {
      cpu.plx(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "RTS" => {
      cpu.rts(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PEI" => {
      cpu.pei(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PLA" => {
      cpu.pla(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "LSR" => {
      cpu.lsr(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "CLC" => {
      cpu.clc(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "CLV" => {
      cpu.clv(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BPL" => {
      cpu.bpl(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "CMP" => {
      cpu.cmp(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BIT" => {
      cpu.bit(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "REP" => {
      cpu.rep(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "STZ" => {
      cpu.stz(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PHA" => {
      cpu.pha(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "MVP" => {
      cpu.mvp(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BRA" => {
      cpu.bra(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "ROR" => {
      cpu.ror(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PLY" => {
      cpu.ply(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "PHX" => {
      cpu.phx(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  "BCC" => {
      cpu.bcc(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }


  &_ => {
    todo!("OP: {} not defined!", op.name);
  }
  }
}

