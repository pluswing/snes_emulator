use log::{debug, info, trace};

use crate::opscodes::{call, CPU_OPS_CODES};

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Accumulator,
    Immediate,
    Absolute,
    Absolute_Indexed_by_X, // Absolute Indexed by X
    Absolute_Indexed_by_Y, // Absolute Indexed by Y
    Implied,

    // 65816
    Absolute_Long, // Absolute Long
    Direct_Page, // Direct Page
    Direct_Page_Indirect, // Direct Page Indirect
    Direct_Page_Indirect_Long, // Direct Page Indirect Long
    Absolute_Long_Indexed_by_X, // Absolute Long Indexed by X
    Direct_Page_Indexed_by_X, // Direct Page Indexed by X
    Direct_Page_Indexed_by_Y, // Direct Page Indexed by Y
    Direct_Page_Indexed_Indirect_by_X, // Direct Page Indexed Indirect by X
    Direct_Page_Indirect_Indexed_by_Y, // Direct Page Indirect Indexed by Y
    Direct_Page_Indirect_Long_Indexed_by_Y, // Direct Page Indirect Long Indexed by Y
    Stack_Relative, // Stack Relative
    Stack_Relative_Indirect_Indexed_by_Y, // Stack Relative Indirect Indexed by Y

    NoneAddressing,

    // added
    Program_Counter_Relative, // Program Counter Relative
    Program_Counter_Relative_Long, // Program Counter Relative Long
    Stack, // = Immediate
    Block_Move, // Block Move

    // TODO ??
    Absolute_Indirect, // Absolute Indirect
    Absolute_Indexed_Indirect, // Absolute Indexed Indirect => Absolute Indexed Indirect, X ??
    Absolute_Indirect_Long // Absolute Indirect Long
}

#[derive(Debug, Clone)]
pub struct OpInfo {
    pub bytes: u16,
    pub cycles: u8,
}

impl OpInfo {
    pub fn new(bytes: u16, cycles: u8) -> Self {
        Self {
            bytes,
            cycles,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpCode {
    pub code: u8,
    pub name: String,
    pub native: OpInfo,
    pub emulation: OpInfo,
    pub addressing_mode: AddressingMode,
}

impl OpCode {
    pub fn new(
        code: u8,
        name: &str,
        native: OpInfo,
        emulation: OpInfo,
        addressing_mode: AddressingMode,
    ) -> Self {
        OpCode {
            code: code,
            name: String::from(name),
            native,
            emulation,
            addressing_mode: addressing_mode,
        }
    }
}

// 1bit	7	6	5	4	3	2	1	0
//      N	V	M	X	D	I	Z	C
//                    E
// N : ネガティブフラグ (1 = Negative)
// V : オーバーフローフラグ (1 = Overflow)
// M : メモリ/アキュームレータ選択フラグ (1 = 8-bit, 0 = 16 bit)
// X : インデックスレジスタ選択フラグ (1 = 8-bit, 0 = 16-bit)
// D : 10進モードフラグ (1 = Decimal, 0 = Binary)
// I : IRQ 禁止フラグ (1 = Disabled)
// Z : ゼロフラグ (1 = Result Zero)
// C : キャリーフラグ (1 = Carry)
// E : エミュレーションフラグ (0 = Native Mode)

const FLAG_NEGATIVE: u8 = 1 << 7;
const FLAG_OVERFLOW: u8 = 1 << 6;
pub const FLAG_MEMORY_ACCUMULATOR_MODE: u8 = 1 << 5;
const FLAG_BREAK2: u8 = 1 << 5; // TODO 使わなければ削除
const FLAG_BREAK: u8 = 1 << 4;
const FLAG_INDEX_REGISTER_MODE: u8 = 1 << 4;
const FLAG_DECIMAL: u8 = 1 << 3;
const FLAG_INTERRRUPT: u8 = 1 << 2;
const FLAG_ZERO: u8 = 1 << 1;
const FLAG_CARRY: u8 = 1 << 0;

pub const MODE_16BIT: u8 = 0;
const MODE_8BIT: u8 = 1;

pub struct CPU {
    pub register_a: u16, // u8モードの時もあり。
    pub register_x: u16,
    pub register_y: u16,
    pub program_counter: u16,
    pub stack_pointer: u16, // u8モードの時もあり。
    pub status: u8,
    pub direct_page: u16, // ダイレクトページレジスタ (D)
    pub data_bank: u8, // データバンクレジスタ (DBR)
    pub program_bank: u8, // プログラムバンクレジスタ (PBR)
    pub mode: u8, // E : エミュレーションフラグ (0 = Native Mode)
    pub memory: Vec<u8>, // size=0xFFFFFF
    // pub bus: Bus<'a>,
    current_op: OpCode,
}

pub static mut IN_TRACE: bool = false;
/*
impl Mem for CPU {
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }
}
*/

impl CPU {

    // for TEST
    pub fn mem_read(&mut self, addr: u32) -> u8 {
        self.memory[addr as usize]
    }
    pub fn mem_write(&mut self, addr: u32, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn new() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            program_counter: 0,
            status: FLAG_INTERRRUPT | FLAG_BREAK2,
            stack_pointer: 0xFD,
            direct_page: 0,
            data_bank: 0,
            program_bank: 0,
            mode: MODE_16BIT,
            memory: vec![0; 0x100_0000],
            // bus: bus,
            current_op: OpCode::new(
              0,
              "",
              OpInfo::new(0, 0),
              OpInfo::new(0, 0),
              AddressingMode::Absolute),
        }
    }

    pub fn pc(&self) -> u32 {
      (self.program_bank as u32) << 16 | self.program_counter as u32
    }

    pub fn is_native_mode(&self) -> bool {
      self.mode == MODE_16BIT
    }

    pub fn is_emulation_mode(&self) -> bool {
      !self.is_native_mode()
    }

    pub fn is_accumulator_16bit_mode(&self) -> bool {
      self.is_native_mode() && (self.status & FLAG_MEMORY_ACCUMULATOR_MODE) == MODE_16BIT
    }


    pub fn is_index_register_16bit_mode(&self) -> bool {
      self.is_native_mode() && (self.status & FLAG_INDEX_REGISTER_MODE) == MODE_16BIT
    }

    pub fn set_register_a(&mut self, value: u16) {
      if self.is_accumulator_16bit_mode() {
        self.register_a = value
      } else {
        self.register_a = (self.register_a & 0xFF00) | (value & 0x00FF)
      }
    }

    pub fn get_register_a(&mut self) -> u16 {
      let a = if self.is_accumulator_16bit_mode() {
        self.register_a
      } else {
        self.register_a & 0x00FF
      };
      a
    }

    pub fn set_register_c(&mut self, value: u16) {
      self.register_a = value
    }

    pub fn get_register_c(&mut self) -> u16 {
      self.register_a
    }

    pub fn set_register_x(&mut self, value: u16) {
      if self.is_index_register_16bit_mode() {
        self.register_x = value
      } else {
        self.register_x = (self.register_x & 0xFF00) | (value & 0x00FF)
      }
    }

    pub fn get_register_x(&mut self) -> u16 {
      if self.is_index_register_16bit_mode() {
        self.register_x
      } else {
        self.register_x & 0x00FF
      }
    }

    pub fn set_register_y(&mut self, value: u16) {
      if self.is_index_register_16bit_mode() {
        self.register_y = value
      } else {
        self.register_y = (self.register_y & 0xFF00) | (value & 0x00FF)
      }
    }

    pub fn get_register_y(&mut self) -> u16 {
      if self.is_index_register_16bit_mode() {
        self.register_y
      } else {
        self.register_y & 0x00FF
      }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u32 {
        let pc = self.pc();
        match mode {
            AddressingMode::Immediate | AddressingMode::Stack => {
              // アドレス部の内容が目的のデータとして利用されます。先頭に#をつけて#$1234のように表します。
              pc
            },
            AddressingMode::Block_Move => {
              // 下位8bitにsrc bank, 上位8bitにdest bankが入ります。
              // MVN srds
              self.mem_read_u16(pc) as u32
            }
            AddressingMode::Absolute => {
              // アドレス部の内容が目的のデータが格納されている16bitアドレスを表します。$1234のように表します。
              let addr = self.wrapped_mem_read_u16(pc) as u32;
              ((self.data_bank as u32) << 16) | addr
            }
            AddressingMode::Absolute_Long => {
              // アドレス部の内容が目的のデータが格納されている24bitフルアドレスを表します。$123456のように表します。
              let addr = if self.is_native_mode() {
                self.wrapped_mem_read_u16(pc)
              } else {
                self.mem_read_u16(pc)
              };
              let bank = self.mem_read(pc & 0xFF0000 | (pc + 2) & 0x00FFFF);
              (bank as u32) << 16 | addr as u32
            }
            AddressingMode::Absolute_Indexed_by_X => {
                // アドレス部の内容にインデクスレジスタの値を足したアドレスが目的のデータが格納されているアドレスを表します。
                // Xレジスタを足すのかYレジスタを足すのかで$1234,xと$1234,yという表し方があります。
                let base = self.mem_read_u16(pc);
                // println!("BASE {:06X}", base);
                let addr = ((self.data_bank as u32) << 16) | base as u32;
                // println!("+DBR {:06X}", addr);
                let addr = addr.wrapping_add(self.get_register_x() as u32);
                // println!("+X {:06X}", addr);
                addr & 0xFFFFFF
            }
            AddressingMode::Absolute_Indexed_by_Y => {
                let base = self.wrapped_mem_read_u16(pc);
                let addr = ((self.data_bank as u32) << 16) | base as u32;
                let addr = addr.wrapping_add(self.get_register_y() as u32);
                addr & 0xFFFFFF
            }
            AddressingMode::Absolute_Long_Indexed_by_X => {
                // アドレス部の内容にインデクスレジスタの値を足したアドレスが目的のデータが格納されている24bitフルアドレスを表します。
                // $123456,xと表します。絶対ロングアドレスインデクスYモードはありません。
                let base = self.wrapped_mem_read_u16(pc);
                let bank = self.mem_read((pc & 0xFF0000) | ((pc + 2) & 0x00FFFF));
                let addr = ((bank as u32) << 16) | base as u32;
                let addr = addr.wrapping_add(self.get_register_x() as u32);
                addr & 0xFFFFFF
            }
            AddressingMode::Absolute_Indirect => {
              // アドレス部のアドレスから16bitを読み込み、それを下位16bit、上位8bitをプログラムバンクレジスタとしたアドレスに目的のデータが格納されています。
              // ($1234)のように表します。
              let base = self.wrapped_mem_read_u16(pc) as u32;
              let base = self.wrapped_mem_read_u16(base) as u32;
              let bank = self.program_bank as u32;
              (bank << 16) | base
            }
            AddressingMode::Absolute_Indexed_Indirect => {
              // アドレス部の内容にXレジスタを足したアドレスから16bitを読み込み、それを下位16bit、上位8bitをプログラムバンクレジスタとしたアドレスに目的のデータが格納されています。
              // 絶対インデクスYインダイレクトモードはありません。($1234, x)のように表します。
              let base = self.wrapped_mem_read_u16(pc);
              let base = base.wrapping_add(self.get_register_x()) & 0x00FFFF;
              let addr = ((self.program_bank as u32) << 16) | base as u32;
              let addr = self.wrapped_mem_read_u16(addr & 0xFFFFFF);
              addr as u32
            }
            AddressingMode::Absolute_Indirect_Long => {
              // アドレス部のアドレスから24bitを読み込んだそのアドレスに目的のデータが格納されています。
              // [$1234]のように表します。
              let base = self.wrapped_mem_read_u16(pc) as u32;
              let addr = self.wrapped_mem_read_u16(base) as u32;
              let bank = self.mem_read((base + 2) & 0x00FFFF);
              (bank as u32) << 16 | addr
            }
            AddressingMode::Direct_Page => {
              // アドレス部の内容にダイレクトページレジスタの値を足したアドレスが目的のデータの各のされているアドレスを表します。
              // $12のように表します。なお、フルアドレス上位8bitは0固定となります。
              let addr = self.mem_read(pc) as u32;
              (self.direct_page as u32).wrapping_add(addr) & 0x00FFFF
            },
            AddressingMode::Direct_Page_Indexed_by_X => {
              // アドレス部の内容にダイレクトページレジスタの値とインデクスレジスタを足したアドレスが目的のデータの各のされているアドレスを表します。
              // Xレジスタを足すのかYレジスタを足すのかで$12,xと$12,yという表し方があります。
              let addr: u32 = self.mem_read(pc) as u32;
              let dp = self.direct_page as u32;
              let addr = dp.wrapping_add(addr);
              let addr = if (dp & 0x00FF) == 0x00 && self.is_emulation_mode() {
                (addr & 0xFF00) | ((addr + self.get_register_x() as u32) & 0x00FF)
              } else {
                addr.wrapping_add(self.get_register_x() as u32)
              };
              addr & 0x00FFFF
            }
            AddressingMode::Direct_Page_Indexed_by_Y => {
              let addr = self.mem_read(pc) as u32;
              let dp = self.direct_page as u32;
              let addr = dp.wrapping_add(addr);
              let addr = if (dp & 0x00FF) == 0x00 && self.is_emulation_mode() {
                (addr & 0xFF00) | ((addr + self.get_register_y() as u32) & 0x00FF)
              } else {
                addr.wrapping_add(self.get_register_y() as u32)
              };
              addr & 0x00FFFF
            }
            AddressingMode::Direct_Page_Indirect => {
              // アドレス部の内容にダイレクトページレジスタの値を足して得られるアドレスから16bitを読み込み、それを下位16bit、DBレジスタを上位8bitとしたアドレスに目的のデータが格納されています。
              // ($12)のように表します。
              let addr = self.mem_read(pc) as u32;
              let addr = (self.direct_page as u32).wrapping_add(addr) & 0x00FFFF;
              ((self.data_bank as u32) << 16) | self.mem_read_u16(addr) as u32
            }
            AddressingMode::Direct_Page_Indirect_Long => {
              // アドレス部の内容にダイレクトページレジスタの値を足して得られるアドレスから24bitを読み込んだそのアドレスに目的のデータが格納されています。
              // [$12]のように表します。
              let base = self.mem_read(pc) as u32;
              let base = (self.direct_page as u32).wrapping_add(base) & 0x00FFFF;
              let addr = self.wrapped_mem_read_u16(base) as u32;
              let bank = self.mem_read((base + 2) & 0x00FFFF);
              ((bank as u32) << 16) | addr as u32
            }
            AddressingMode::Direct_Page_Indexed_Indirect_by_X => {
              // アドレス部の内容にダイレクトページレジスタとXレジスタの値を足して得られるアドレスから16bitを読み込み、それを下位16bit、DBレジスタを上位8bitとしたアドレスに目的のデータが格納されています。
              // ダイレクトインデクスYインダイレクトモードはありません。($12, x)のように表します。
              let addr: u32 = self.mem_read(pc) as u32;
              let dp = self.direct_page as u32;
              let addr = dp.wrapping_add(addr);
              let addr = if (dp & 0x00FF) == 0x00 && self.is_emulation_mode() {
                (addr & 0xFF00) | ((addr + self.get_register_x() as u32) & 0x00FF)
              } else {
                addr.wrapping_add(self.get_register_x() as u32)
              };
              let addr = addr & 0x00FFFF;
              let addr = self.wrapped_mem_read_u16(addr) as u32;
              (self.data_bank as u32) << 16 | addr
            }
            // LDA (dp), Y => B1 dp
            AddressingMode::Direct_Page_Indirect_Indexed_by_Y => {
              // アドレス部の内容にダイレクトページレジスタの値を足して得られるアドレスから16bitを読み込み、さらにそれにYレジスタを足したものを下位16bit、DBレジスタを上位8bitとしたアドレスに目的のデータが格納されています。
              // ($12),yのように表します。
              let addr = self.mem_read(pc);
              let addr = (self.direct_page as u32).wrapping_add(addr as u32);
              let addr = addr & 0x00FFFF;
              let addr = self.wrapped_mem_read_u16(addr) as u32;
              let addr = addr.wrapping_add(self.get_register_y() as u32);
              ((self.data_bank as u32) << 16).wrapping_add(addr) & 0xFFFFFF
            }
            AddressingMode::Direct_Page_Indirect_Long_Indexed_by_Y => {
              // アドレス部の内容にダイレクトページレジスタの値を足して得られるアドレスから24bitを読み込み、さらにYレジスタを足したアドレスに目的のデータが格納されています。
              // ダイレクトインダイレクトロングインデクスXモードはありません。[$12],yのように表します。
              let addr = self.mem_read(pc);
              let addr = (self.direct_page as u32).wrapping_add(addr as u32);
              let addr = addr & 0x00FFFF;
              let base = self.wrapped_mem_read_u16(addr);
              // println!("ADDR: {:06X}", addr);
              let bank = if (self.direct_page & 0x00FF) == 0x00 && self.is_emulation_mode() {
                let addr = addr & 0xFF00 | (addr + 2) & 0x00FF;
                self.mem_read(addr)
              } else {
                self.mem_read((addr + 2) & 0xFFFF)
              };
              // println!("BANK: {:06X}", bank);
              let addr = ((bank as u32) << 16) | base as u32;
              // println!("ADDR: {:06X}", addr);
              let addr: u32 = addr.wrapping_add(self.get_register_y() as u32);
              // println!("ADDR: {:06X} BASE: {:04X}", addr & 0xFFFFFF, base);
              addr & 0xFFFFFF
            }
            AddressingMode::Program_Counter_Relative => {
                // ソース上の現在位置と、目的データの対象の位置との差分でアドレスを指定します。プログラムカウンタにアドレス部の内容を足したものが目的のデータのアドレスとなります。参照範囲は-128バイト～+127バイト以内でなければいけません。アセンブリコード上では基本的にラベルを用いて記述します(つまりアセンブル時に自動的にコード配置がされる)。
                let base = self.mem_read(pc);
                let np = (base as i8) as i32 + pc as i32;
                return np as u32;
            }
            AddressingMode::Program_Counter_Relative_Long => {
                // ソース上の現在位置と、目的データの対象の位置との差分でアドレスを指定します。プログラムカウンタにアドレス部の内容を足したものが目的のデータのアドレスとなります。参照範囲は-32768バイト～+32767バイト以内でなければいけません。アセンブリコード上では基本的にラベルを用いて記述します(つまりアセンブル時に自動的にコード配置がされる)。
                let base: u16 = self.mem_read_u16(pc);
                let np = (base as i16) as i32 + pc as i32;
                return np as u32;
            }
            AddressingMode::Stack_Relative => {
              // アドレス部の内容にスタックポインタを足したアドレスが目的のデータが格納されたアドレスを表します。
              // スタックポインタは常に次の有効なスタックの空き領域を示しているため、オペランドに1を指定すれば最後にスタックに積まれた値、0を指定すれば最後にスタックからプルされた値を指す。
              // $01,sのように表します。
              let value = self.mem_read(pc) as u32;
              let addr = (self.stack_pointer as u32).wrapping_add(value);
              println!("{:04X}, {:06X}", value, addr);
              addr & 0x00FFFF
            }
            AddressingMode::Stack_Relative_Indirect_Indexed_by_Y => {
              // アドレス部の内容にスタックポインタを足したアドレスから16bitを読み込み、さらにYレジスタを足したアドレスが目的のデータの格納されているアドレスを表します。
              // ($01,s),yのように表します。
              let value = self.mem_read(pc) as u32;
              let addr = (self.stack_pointer as u32).wrapping_add(value);
              let addr = addr & 0x00FFFF;
              let addr = self.wrapped_mem_read_u16(addr) as u32;
              let addr = addr.wrapping_add(self.get_register_y() as u32);
              let addr = ((self.data_bank as u32) << 16).wrapping_add(addr);
              println!("ADDR: {:06X}", addr);
              addr & 0xFFFFFF
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
            _ => {
              panic!("mode {:?} is not supported", mode);
            }
        }
    }

    pub fn mem_read_u16(&mut self, pos: u32) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    pub fn wrapped_mem_read_u16(&mut self, pos: u32) -> u16 {
      if (self.direct_page & 0x00FF) == 0x00 && self.is_emulation_mode() && self.current_op.name != "TSB" {
          let lo = self.mem_read(pos) as u16;
          let hi = self.mem_read((pos & 0xFFFF00) | ((pos + 1) & 0x0000FF))  as u16;
          (hi << 8) | (lo as u16)
        } else {
          let lo = self.mem_read(pos) as u16;
          let hi = self.mem_read((pos & 0xFF0000) | ((pos + 1) & 0x00FFFF))  as u16;
          (hi << 8) | (lo as u16)
        }
    }

    pub fn mem_write_u16(&mut self, pos: u32, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0x00FF) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn mem_read_auto(&mut self, pos: u32) -> u16 {
        let lo = self.mem_read(pos) as u16;
        if self.is_accumulator_16bit_mode() {
          let hi = self.mem_read(pos + 1) as u16;
          (hi << 8) | (lo as u16)
        } else {
          lo
        }
    }

    pub fn mem_write_auto(&mut self, pos: u32, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0x00FF) as u8;
        self.mem_write(pos, lo);
        if self.is_accumulator_16bit_mode() {
          self.mem_write(pos + 1, hi);
        }
    }

    fn load_and_run(&mut self, program: Vec<u8>) {
        self.load();
        self.reset();
        self.run();
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        // FIXME あってる？
        self.status = FLAG_INTERRRUPT | FLAG_BREAK2;
        self.stack_pointer = 0xFD;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self) {
        // self.mem_write_u16(0xFFFC, 0x8000);
    }

    fn apply_mode(&mut self, force: bool) {
      if self.is_emulation_mode() {
        if force {
          self.stack_pointer = 0x0100 | (self.stack_pointer & 0x00FF);
          return
        }

        // たぶん、
        // 新しい命令(6502にない命令)は、無視。
        // 新しいアドレシングモード(LONG)も無視するっぽい。
        if self.current_op.name == "RTL" {
          return
        }
        if self.current_op.name == "PEA" {
          return
        }

        // AddressingMode::Absolute_LongはJSR命令のテストで必要だったので追加。
        if self.current_op.addressing_mode != AddressingMode::Absolute_Long {
          self.stack_pointer = 0x0100 | (self.stack_pointer & 0x00FF);
        }
      }
    }

    pub fn run(&mut self) {
        // if let Some(_nmi) = self.bus.poll_nmi_status() {
        //     self.interrupt_nmi();
        // }

        // if self.bus.poll_apu_irq() {
        //     self.apu_irq();
        // } else if unsafe { MAPPER.is_irq() } {
        //     self.apu_irq();
        // }

        let pc = (self.program_bank as u32) << 16 | self.program_counter as u32;
        let opscode = self.mem_read(pc);
        self.program_counter = self.program_counter.wrapping_add(1);

        let op = CPU_OPS_CODES.get(&opscode);
        self.current_op = op.unwrap().clone();
        self.apply_mode(true);
        match op {
            Some(op) => {

                call(self, &op);

                // self.bus.tick(op.cycles + self.add_cycles);

                // if program_conter_state == self.program_counter {
                //   self.program_counter += (op.len - 1) as u16
                // }
            }
            _ => {} // panic!("no implementation {:<02X}", opscode),
        }
        self.apply_mode(true);
    }
/* ファミコンの割り込み実装。
    fn interrupt_nmi(&mut self) {
        debug!("** INTERRUPT_NMI **");
        self._push_u16(self.program_counter);
        let mut status = self.status;
        status = status & !FLAG_BREAK;
        status = status | FLAG_BREAK2;
        self._push(status);

        self.status = self.status | FLAG_INTERRRUPT;
        self.bus.tick(2);
        self.program_counter = self.mem_read_u16(0xFFFA);
    }

    fn apu_irq(&mut self) {
        if self.status & FLAG_INTERRRUPT != 0 {
            return;
        }
        self._push_u16(self.program_counter);
        self._push(self.status);
        self.program_counter = self.mem_read_u16(0xFFFE);
        self.status = self.status | FLAG_BREAK;
    }
*/

    pub fn phx(&mut self, mode: &AddressingMode) {
      let x = self.get_register_x();
      if self.is_index_register_16bit_mode() {
        self._push_u16(x);
      } else {
        self._push(x as u8);
      }
    }
    pub fn ply(&mut self, mode: &AddressingMode) {
      let y = if self.is_index_register_16bit_mode() {
        self._pop_u16()
      } else {
        self._pop() as u16
      };
      self.set_register_y(y);
      self.update_zero_and_negative_flags_xy(y);
    }
    pub fn bra(&mut self, mode: &AddressingMode) {
      self._branch(mode, 0x00, false);
    }
    pub fn mvp(&mut self, mode: &AddressingMode) {
      let value = self.get_operand_address(mode);
      let src_bank = (value & 0xFF00) >> 8;
      let dest_bank = value & 0x00FF;

      let copy_bytes = self.get_register_c() as u32 + 1;

      // FIXME! for test
      // let copy_bytes = if copy_bytes > 0x0E { 0x0E } else { copy_bytes };
      // println!("COPY_BYTES: {:04X}", copy_bytes);

      for i in 0..copy_bytes {
        let src_addr = (src_bank << 16) | self.get_register_x() as u32;
        let dest_addr = (dest_bank << 16) | self.get_register_y() as u32;

        let val = self.mem_read(src_addr);
        self.mem_write(dest_addr, val);

        let x = self.get_register_x();
        self.set_register_x(x.wrapping_sub(1));

        let y: u16 = self.get_register_y();
        self.set_register_y(y.wrapping_sub(1));

        let c = self.get_register_c();
        self.set_register_c(c.wrapping_sub(1));
      }

      self.data_bank = dest_bank as u8;
    }

    pub fn stz(&mut self, mode: &AddressingMode) {
      let addr = self.get_operand_address(mode);
      self.mem_write_auto(addr, 0);
    }

    pub fn rep(&mut self, mode: &AddressingMode) {
      let addr = self.get_operand_address(mode);
      let value = self.mem_read(addr);

      if self.is_native_mode() {
        self.status = self.status & !value;
      } else {
        // = FLAG_BREAK, FLAG_BREAK2
        let mask = !(FLAG_MEMORY_ACCUMULATOR_MODE | FLAG_INDEX_REGISTER_MODE);
        self.status = ((self.status & !value) & mask)
          | (self.status & FLAG_MEMORY_ACCUMULATOR_MODE)
          | (self.status & FLAG_INDEX_REGISTER_MODE);
      }
    }
    pub fn pei(&mut self, mode: &AddressingMode) {
      let addr = self.get_operand_address(mode);
      let addr = self.mem_read(addr);
      let addr = self.direct_page.wrapping_add(addr as u16) & 0x00FFFF;
      let value = self.wrapped_mem_read_u16(addr as u32);
      self._push_u16(value as u16);
    }
    pub fn plx(&mut self, mode: &AddressingMode) {
      let x = if self.is_index_register_16bit_mode() {
        self._pop_u16()
      } else {
        self._pop() as u16
      };
      self.set_register_x(x);
      self.update_zero_and_negative_flags_xy(x);
    }
    pub fn phy(&mut self, mode: &AddressingMode) {
      let y = self.get_register_y();
      if self.is_index_register_16bit_mode() {
        self._push_u16(y);
      } else {
        self._push(y as u8);
      }
    }
    pub fn wdm(&mut self, mode: &AddressingMode) {
        todo!("wdm")
    }
    pub fn cop(&mut self, mode: &AddressingMode) {
      // FIXME
      // program bank register (ネイティブモードの時。)
      // program counter
      // status register をスタックに積む
      //  エミュレーションモード => pc = 0xFFF4の内容
      //  ネイティブモード      => pc = $00:FFE4の内容にする。
      if self.is_native_mode() {
        self._push(self.program_bank);
      }
      // 仕様上は+2。run()で+1しているので、ここでは+1でOK。
      self._push_u16(self.program_counter + 1);
      self._push(self.status);
      self.program_bank = 0x00;
      if self.is_native_mode() {
        self.program_counter = self.mem_read_u16(0x00FFE4);
      } else {
        self.program_counter = self.mem_read_u16(0xFFF4);
      }
      self.status = self.status | FLAG_INTERRRUPT;
      self.status = self.status & !FLAG_DECIMAL;
      // opscodes.rsで自動加算されるため、調整。
      self.program_counter = self.program_counter.wrapping_sub(1);
    }
    pub fn brl(&mut self, mode: &AddressingMode) {
        self._branch(mode, 0x00, false);
    }
    pub fn tdc(&mut self, mode: &AddressingMode) {
        self.register_a = self.direct_page;
        self._update_zero_and_negative_flags(self.register_a, true);
    }
    pub fn phk(&mut self, mode: &AddressingMode) {
        self._push(self.program_bank);
    }
    pub fn tcd(&mut self, mode: &AddressingMode) {
        self.direct_page = self.register_a;
        self._update_zero_and_negative_flags(self.register_a, true);
    }
    pub fn stp(&mut self, mode: &AddressingMode) {
      // 何もしなくて良さそう。
      // TODO 次の命令が実行されなくする必要があるかも。
    }
    pub fn mvn(&mut self, mode: &AddressingMode) {
      // 連続したメモリブロックをコピーする
      // Xインデックスレジスタは、ブロックの開始（最小）ソースアドレスを指定します。
      // Yインデックスレジスタは、ブロックの開始（最小）宛先アドレスを指定します。
      // Cアキュムレータ (=Aレジスタ)は、ブロックの長さをバイト単位で指定し、そこから1を引いた値を指定します。
      // 最初のオペランドバイトは、ブロックが格納される宛先バンクを指定します。
      // 2番目のオペランドバイトは、ブロックの開始位置となるソースバンクを指定します。
      let value = self.get_operand_address(mode);
      let src_bank = (value & 0xFF00) >> 8;
      let dest_bank = value & 0x00FF;

      let copy_bytes = self.get_register_c() as u32 + 1;

      // FIXME! for test
      // let copy_bytes = if copy_bytes > 0x0E { 0x0E } else { copy_bytes };
      // println!("COPY_BYTES: {:04X}", copy_bytes);

      for i in 0..copy_bytes {
        let src_addr = (src_bank << 16) | self.get_register_x() as u32;
        let dest_addr = (dest_bank << 16) | self.get_register_y() as u32;

        let val = self.mem_read(src_addr);
        self.mem_write(dest_addr, val);

        let x = self.get_register_x();
        self.set_register_x(x.wrapping_add(1));

        let y: u16 = self.get_register_y();
        self.set_register_y(y.wrapping_add(1));

        let c = self.get_register_c();
        self.set_register_c(c.wrapping_sub(1));
      }

      self.data_bank = dest_bank as u8;
    }
    pub fn xce(&mut self, mode: &AddressingMode) {
        todo!("xce")
    }
    pub fn rtl(&mut self, mode: &AddressingMode) {
      // RTLはスタックから戻りアドレスを取得しますが、プログラムカウンタにロードする前に値を1つ増やします。
      // 次に、呼び出し元のバンクがプログラムバンクレジスタに読み込まれます。つまり、RTLはJSLがスタックに対して行った操作を元に戻すのです。
      self.program_counter = self._pop_u16().wrapping_add(1);
      self.program_bank = self._pop();
    }
    pub fn sep(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.status = self.status | value;
        self.clear_index_register_upper_byte();
    }

    pub fn tsb(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read_auto(addr);
        let a = self.get_register_a();
        let result = data | a;
        self.mem_write_auto(addr, result);

        let result = data & a;
        // println!("A: {:04X}, M: {:04X}, R: {:04X}", a, data, result);
        if result == 0 {
          self.status = self.status | FLAG_ZERO;
        } else {
          self.status = self.status & (!FLAG_ZERO);
        }
    }

    pub fn pld(&mut self, mode: &AddressingMode) {
      self.direct_page = self._pop_u16();
      self._update_zero_and_negative_flags(self.direct_page, true);
    }
    pub fn tcs(&mut self, mode: &AddressingMode) {
        self.stack_pointer = if self.is_native_mode() {
          self.register_a
        } else {
          self.register_a & 0x00FF
        };
    }
    pub fn xba(&mut self, mode: &AddressingMode) {
        todo!("xba")
    }
    pub fn phd(&mut self, mode: &AddressingMode) {
        self._push_u16(self.direct_page);
    }
    pub fn tsc(&mut self, mode: &AddressingMode) {
        self.register_a = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_a);
    }
    pub fn tyx(&mut self, mode: &AddressingMode) {
        let y = self.get_register_y();
        self.set_register_x(y);
        self.update_zero_and_negative_flags_xy(y);
    }
    pub fn pea(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read_u16(addr);
        self._push_u16(value as u16);
    }
    pub fn wai(&mut self, mode: &AddressingMode) {
        todo!("wai")
    }
    pub fn txy(&mut self, mode: &AddressingMode) {
        let x = self.get_register_x();
        self.set_register_y(x);
        self.update_zero_and_negative_flags_xy(x);
    }
    pub fn trb(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read_auto(addr);
        let a = self.get_register_a();
        let result = data & !a;
        self.mem_write_auto(addr, result);

        let result = data & a;
        if result == 0 {
          self.status = self.status | FLAG_ZERO;
        } else {
          self.status = self.status & (!FLAG_ZERO);
        }
    }
    pub fn plb(&mut self, mode: &AddressingMode) {
      self.data_bank = self._pop();
      self._update_zero_and_negative_flags(self.data_bank as u16, false);
    }
    pub fn per(&mut self, mode: &AddressingMode) {
      let pc = self.get_operand_address(mode);
      let value = self.mem_read_u16(pc);
      let arg_bytes = self.current_op.native.bytes - 1;
      let pc = pc + arg_bytes as u32;
      let value = (pc + value as u32) & 0x00FFFF;
      self._push_u16(value as u16);
    }
    pub fn phb(&mut self, mode: &AddressingMode) {
      self._push(self.data_bank);
    }

    pub fn anc(&mut self, mode: &AddressingMode) {
        todo!("anc")
    }
    pub fn arr(&mut self, mode: &AddressingMode) {
        todo!("arr")
    }
    pub fn asr(&mut self, mode: &AddressingMode) {
        todo!("asr")
    }
    pub fn lxa(&mut self, mode: &AddressingMode) {
        todo!("lxa")
    }
    pub fn sha(&mut self, mode: &AddressingMode) {
        todo!("sha")
    }
    pub fn sbx(&mut self, mode: &AddressingMode) {
        //  A&X minus #{imm} into X
        // AND X register with accumulator and store result in X regis-ter, then
        // subtract byte from X register (without borrow).
        // Status flags: N,Z,C

        // AND X をアキュムレータに登録し、結果を X レジスタに格納します。 X レジスタからバイトを減算します (ボローなし)。 ステータスフラグ：N、Z、C
        /*
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let (v, overflow) = (self.register_a & self.register_x).overflowing_sub(value);
        self.register_x = v;
        self.update_zero_and_negative_flags(self.register_x);
        self.status = if overflow {
            self.status & FLAG_OVERFLOW
        } else {
            self.status | FLAG_OVERFLOW
        };
        */
        todo!("sbx")
    }

    pub fn jam(&mut self, mode: &AddressingMode) {
        // Stop program counter (processor lock up).
        self.program_counter -= 1;
        panic!("CALL JAM operation.");
    }

    pub fn lae(&mut self, mode: &AddressingMode) {
        // stores {adr}&S into A, X and S

        // AND memory with stack pointer, transfer result to accu-mulator, X
        // register and stack pointer.
        // Status flags: N,Z
        /*
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let s = self._pop();
        self.register_a = value & s;
        self.register_x = self.register_a;
        self._push(self.register_a);
        self.update_zero_and_negative_flags(self.register_a);
        */
        todo!("lae")
    }

    pub fn shx(&mut self, mode: &AddressingMode) {
        // M =3D X AND HIGH(arg) + 1
        /*
        let addr = self.get_operand_address(mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, (self.register_x & h).wrapping_add(1));
        */
        todo!("shx")
    }

    pub fn shy(&mut self, mode: &AddressingMode) {
        // Y&H into {adr}
        // AND Y register with the high byte of the target address of the argument
        // + 1. Store the result in memory.
        /*
        let addr = self.get_operand_address(mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, (self.register_y & h).wrapping_add(1));
        */
        todo!("shy")
    }

    pub fn ane(&mut self, mode: &AddressingMode) {
        // TXA + AND #{imm}
        self.txa(mode);
        self.and(mode);
        todo!("ane")
    }

    pub fn shs(&mut self, mode: &AddressingMode) {
        // stores A&X into S and A&X&H into {adr}
        // アキュムレータと X レジスタを AND 演算し、結果をスタック ポインタに格納します。次に、スタック ポインタと引数 1 のターゲット アドレスの上位バイトを AND 演算します。結果をメモリに格納します。
        /*
        self._push(self.register_a & self.register_x);
        let addr = self.get_operand_address(mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, self.register_a & self.register_x & h);
        */
        todo!("shs")
    }

    pub fn rra(&mut self, mode: &AddressingMode) {
        self.ror(mode);
        self.adc(mode);
    }

    pub fn sre(&mut self, mode: &AddressingMode) {
        self.lsr(mode);
        self.eor(mode);
    }

    pub fn rla(&mut self, mode: &AddressingMode) {
        self.rol(mode);
        self.and(mode);
    }

    pub fn slo(&mut self, mode: &AddressingMode) {
        self.asl(mode);
        self.ora(mode);
    }

    pub fn isb(&mut self, mode: &AddressingMode) {
        // = ISC
        self.inc(mode);
        self.sbc(mode);
    }

    pub fn dcp(&mut self, mode: &AddressingMode) {
        self.dec(mode);
        self.cmp(mode);
    }

    pub fn sax(&mut self, mode: &AddressingMode) {
      /*
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a & self.register_x);
      */
      todo!("sax");
    }

    pub fn lax(&mut self, mode: &AddressingMode) {
        self.lda(mode);
        self.tax(mode);
    }

    pub fn txs(&mut self, mode: &AddressingMode) {
        self.stack_pointer = self.get_register_x();
    }

    pub fn tsx(&mut self, mode: &AddressingMode) {
        self.set_register_x(self.stack_pointer);
        let x = self.get_register_x();
        self.update_zero_and_negative_flags_xy(x);
    }

    pub fn tya(&mut self, mode: &AddressingMode) {
        self.set_register_a(self.register_y);
        let a = self.get_register_a();
        self.update_zero_and_negative_flags(a);
    }

    pub fn tay(&mut self, mode: &AddressingMode) {
        self.set_register_y(self.register_a);
        let y = self.get_register_y();
        self.update_zero_and_negative_flags_xy(y);
    }

    pub fn txa(&mut self, mode: &AddressingMode) {
        self.set_register_a(self.register_x);
        let a = self.get_register_a();
        self.update_zero_and_negative_flags(a);
    }

    pub fn tax(&mut self, mode: &AddressingMode) {
        if !self.is_accumulator_16bit_mode() && self.is_index_register_16bit_mode() {
          // この場合は、16bit転送になる。
          self.register_x = self.register_a;
        } else {
          let a = self.get_register_a();
          self.set_register_x(a);
        }
        let x = self.get_register_x();
        self.update_zero_and_negative_flags_xy(x);
    }

    pub fn sty(&mut self, mode: &AddressingMode) {
      let addr = self.get_operand_address(mode);
      let y = self.get_register_y();
      if self.is_native_mode() {
        self.mem_write_u16(addr, y);
      } else {
        self.mem_write(addr, y as u8);
      }
    }

    pub fn stx(&mut self, mode: &AddressingMode) {
      let addr = self.get_operand_address(mode);
      let x = self.get_register_x();
      if self.is_native_mode() {
        self.mem_write_u16(addr, x);
      } else {
        self.mem_write(addr, x as u8);
      }
    }

    pub fn sta(&mut self, mode: &AddressingMode) {
      let addr = self.get_operand_address(mode);
      let a = self.get_register_a();
      self.mem_write_auto(addr, a);
    }

    fn clear_index_register_upper_byte(&mut self) {
      if self.is_index_register_16bit_mode() {
        return;
      }
      self.register_x = self.register_x & 0x00FF;
      self.register_y = self.register_y & 0x00FF;
    }

    pub fn rti(&mut self, mode: &AddressingMode) {
      // スタックからプロセッサ フラグをプルし、続いてプログラム カウンタをプルします。
      if self.is_native_mode() {
        self.status = self._pop();
        self.clear_index_register_upper_byte();
      } else {
        let mask = !(FLAG_MEMORY_ACCUMULATOR_MODE | FLAG_INDEX_REGISTER_MODE);
          self.status = (self._pop() & mask)
            | (self.status & FLAG_MEMORY_ACCUMULATOR_MODE)
            | (self.status & FLAG_INDEX_REGISTER_MODE);
      }
      self.program_counter = self._pop_u16();
      if self.is_native_mode() {
        self.program_bank = self._pop();
      }
    }

    pub fn plp(&mut self, mode: &AddressingMode) {
        if self.is_native_mode() {
          self.status = self._pop();
          self.clear_index_register_upper_byte();
        } else {
          // = FLAG_BREAK, FLAG_BREAK2
          let mask = !(FLAG_MEMORY_ACCUMULATOR_MODE | FLAG_INDEX_REGISTER_MODE);
          self.status = (self._pop() & mask)
            | (self.status & FLAG_MEMORY_ACCUMULATOR_MODE)
            | (self.status & FLAG_INDEX_REGISTER_MODE);
        }
    }

    pub fn php(&mut self, mode: &AddressingMode) {
        // fullsnes は、PHP は常にbreak フラグに 1 を書き込むと主張している。
        self._push(self.status);
    }

    pub fn pla(&mut self, mode: &AddressingMode) {
      let a = if self.is_accumulator_16bit_mode() {
        self._pop_u16()
      } else {
        self._pop() as u16
      };
      self.set_register_a(a);
      self.update_zero_and_negative_flags(a);
    }

    pub fn pha(&mut self, mode: &AddressingMode) {
      let a = self.get_register_a();
      if self.is_accumulator_16bit_mode() {
        self._push_u16(a);
      } else {
        self._push(a as u8);
      }
    }

    pub fn nop(&mut self, mode: &AddressingMode) {
        // なにもしない
    }

    pub fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read_u16(addr);
        self.set_register_y(value);
        let y = self.get_register_y();
        self.update_zero_and_negative_flags_xy(y);
    }

    pub fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read_u16(addr);
        self.set_register_x(value);
        let x = self.get_register_x();
        self.update_zero_and_negative_flags_xy(x);
    }

    pub fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read_u16(addr);
        self.set_register_a(value);
        let a = self.get_register_a();
        self.update_zero_and_negative_flags(a);
    }

    pub fn rts(&mut self, mode: &AddressingMode) {
        self.program_counter = self._pop_u16().wrapping_add(1);
    }

    pub fn jsr(&mut self, mode: &AddressingMode) {
      // opscodes.rsのcall関数内でprogram_counterを変更しないようにする必要あり。
      let bytes = if self.is_accumulator_16bit_mode() {
        self.current_op.native.bytes
      } else {
        self.current_op.emulation.bytes
      };
      let addr = self.get_operand_address(mode);
      match mode {
        AddressingMode::Absolute_Long => {
          self._push(self.program_bank);
          self.program_bank = (addr >> 16) as u8;
        },
        _ => {}
      }
      let run_func_correction = 1;
      let last_byte_correction = 1;
      let pc = self.program_counter as u32 + bytes as u32 - run_func_correction - last_byte_correction;
      self._push_u16(pc as u16);
      self.program_counter = addr as u16;
    }

    pub fn _push(&mut self, value: u8) {
      let addr = self.stack_pointer as u32;
      println!("STACK PUSH: {:04X} => {:02X}", self.stack_pointer, value);
      self.mem_write(addr, value);
      self.stack_pointer = self.stack_pointer.wrapping_sub(1);
      self.apply_mode(false);
    }

    pub fn _pop(&mut self) -> u8 {
      self.stack_pointer = self.stack_pointer.wrapping_add(1);
      self.apply_mode(false);
      let value = self.mem_read(self.stack_pointer as u32);
      println!("STACK POP: {:04X} => {:02X}", self.stack_pointer, value);
      value
    }

    pub fn _push_u16(&mut self, value: u16) {
        self._push((value >> 8) as u8);
        self._push((value & 0x00FF) as u8);
    }

    pub fn _pop_u16(&mut self) -> u16 {
        let lo = self._pop();
        let hi = self._pop();
        ((hi as u16) << 8) | lo as u16
    }

    pub fn jmp(&mut self, mode: &AddressingMode) {
        // opscodes.rsのcall関数内でprogram_counterを変更しないようにする必要あり。
        let addr = self.get_operand_address(mode);
        println!("ADDR: {:06X}", addr);
        self.program_counter = addr as u16;
        match mode {
          AddressingMode::Absolute_Long
          | AddressingMode::Absolute_Indirect_Long => {
          self.program_bank = (addr >> 16) as u8;
          }
          _ => {
          }
        }
    }

    pub fn iny(&mut self, mode: &AddressingMode) {
        let y = self.get_register_y();
        self.set_register_y(y.wrapping_add(1));
        let y = self.get_register_y();
        self.update_zero_and_negative_flags_xy(y);
    }

    pub fn inx(&mut self, mode: &AddressingMode) {
        let x = self.get_register_x();
        self.set_register_x(x.wrapping_add(1));
        let x = self.get_register_x();
        self.update_zero_and_negative_flags_xy(x);
    }

    pub fn inc(&mut self, mode: &AddressingMode) {
      if mode == &AddressingMode::Accumulator {
        let value = self.get_register_a().wrapping_add(1);
        self.set_register_a(value);
        let a = self.get_register_a();
        self.update_zero_and_negative_flags(a);
      } else {
        let addr = self.get_operand_address(mode);
        let value = if self.is_accumulator_16bit_mode() {
          let value = self.mem_read_u16(addr).wrapping_add(1);
          self.mem_write_u16(addr, value);
          value
        } else {
          let value = self.mem_read(addr).wrapping_add(1);
          self.mem_write(addr, value);
          value as u16
        };
        self.update_zero_and_negative_flags(value);
      }
    }

    pub fn dey(&mut self, mode: &AddressingMode) {
        let value = self.get_register_y().wrapping_sub(1);
        self.set_register_y(value);
        self.update_zero_and_negative_flags_xy(value);
    }

    pub fn dex(&mut self, mode: &AddressingMode) {
        let value = self.get_register_x().wrapping_sub(1);
        self.set_register_x(value);
        self.update_zero_and_negative_flags_xy(value);
    }

    pub fn dec(&mut self, mode: &AddressingMode) {
      if mode == &AddressingMode::Accumulator {
        let value = self.get_register_a().wrapping_sub(1);
        self.set_register_a(value);
        let a = self.get_register_a();
        self.update_zero_and_negative_flags(a);
      } else {
        let addr = self.get_operand_address(mode);
        let value = if self.is_accumulator_16bit_mode() {
          let value = self.mem_read_u16(addr).wrapping_sub(1);
          self.mem_write_u16(addr, value);
          value
        } else {
          let value = self.mem_read(addr).wrapping_sub(1);
          self.mem_write(addr, value);
          value as u16
        };
        self.update_zero_and_negative_flags(value);
      }
    }

    pub fn cpy(&mut self, mode: &AddressingMode) {
      let target = self.get_register_y();
      let addr = self.get_operand_address(mode);
      let value = if self.is_index_register_16bit_mode() {
        self.wrapped_mem_read_u16(addr)
      } else {
        self.mem_read(addr) as u16
      };
      if target >= value {
          self.sec(&AddressingMode::Implied);
      } else {
          self.clc(&AddressingMode::Implied);
      }
      let value = target.wrapping_sub(value);
      self.update_zero_and_negative_flags_xy(value);
    }

    pub fn cpx(&mut self, mode: &AddressingMode) {
      let target = self.get_register_x();
      let addr = self.get_operand_address(mode);
      let value = if self.is_index_register_16bit_mode() {
        self.mem_read_u16(addr)
      } else {
        self.mem_read(addr) as u16
      };
      if target >= value {
          self.sec(&AddressingMode::Implied);
      } else {
          self.clc(&AddressingMode::Implied);
      }
      let value = target.wrapping_sub(value);
        self.update_zero_and_negative_flags_xy(value);
    }

    pub fn cmp(&mut self, mode: &AddressingMode) {
      let target = self.get_register_a();
      let addr = self.get_operand_address(mode);
      let value = if self.is_accumulator_16bit_mode() {
        self.mem_read_u16(addr)
      } else {
        self.mem_read(addr) as u16
      };
      if target >= value {
          self.sec(&AddressingMode::Implied);
      } else {
          self.clc(&AddressingMode::Implied);
      }
      let value = target.wrapping_sub(value);
      self.update_zero_and_negative_flags(value);
    }

    pub fn clv(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_OVERFLOW;
    }

    pub fn sei(&mut self, mode: &AddressingMode) {
        self.status = self.status | FLAG_INTERRRUPT;
    }

    pub fn cli(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_INTERRRUPT;
    }

    pub fn sed(&mut self, mode: &AddressingMode) {
        self.status = self.status | FLAG_DECIMAL;
    }

    pub fn cld(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_DECIMAL;
    }

    pub fn sec(&mut self, mode: &AddressingMode) {
        self.status = self.status | FLAG_CARRY;
    }

    pub fn clc(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_CARRY;
    }

    pub fn bvs(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_OVERFLOW, true);
    }

    pub fn bvc(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_OVERFLOW, false);
    }

    fn _branch(&mut self, mode: &AddressingMode, flag: u8, nonzero: bool) {
        let addr = self.get_operand_address(mode);
        if nonzero {
            if self.status & flag != 0 {
                // self.add_cycles += 1;
                // if (self.program_counter & 0xFF00) != (addr & 0xFF00) {
                //     self.add_cycles += 1;
                // }
                self.program_counter = addr as u16
            }
        } else {
            if self.status & flag == 0 {
                // (+1 if branch succeeds
                //  +2 if to a new page)
                // self.add_cycles += 1;
                // if (self.program_counter & 0xFF00) != (addr & 0xFF00) {
                //     self.add_cycles += 1;
                // }
                self.program_counter = addr as u16
            }
        }
    }

    pub fn brk(&mut self, mode: &AddressingMode) {

        // ネイティブモードでは、プログラムバンクレジスタがスタックにプッシュされます。次に、プログラムカウンタが2インクリメントされ、スタックにプッシュされます。次に、ステータスレジスタ（エミュレーションモードの場合はブレークフラグがセットされた状態）がスタックにプッシュされます。次に、割り込み無効フラグがセットされます。ネイティブモードでは、プログラムバンクレジスタがクリアされます。

        // FLAG_BREAK が立っている場合は
        if self.status & FLAG_BREAK != 0 {
            return;
        }

        // プログラム カウンターとプロセッサ ステータスがスタックにプッシュされ、
        self._push_u16(self.program_counter + 1);
        self._push(self.status);

        // $FFFE/F の IRQ 割り込みベクトルが PC にロードされ、ステータスのブレーク フラグが 1 に設定されます。
        self.program_counter = self.mem_read_u16(0xFFFE);
        self.status = self.status | FLAG_BREAK;
    }

    pub fn bpl(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_NEGATIVE, false);
    }

    pub fn bmi(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_NEGATIVE, true);
    }

    pub fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read_u16(addr);
        println!("ADDR: {:06X} VALUE: {:06X}", addr, value);

        let zero = self.get_register_a() & value;
        if zero == 0 {
            self.status = self.status | FLAG_ZERO;
        } else {
            self.status = self.status & !FLAG_ZERO;
        }
        // Immediateの場合は、nagative, overflowフラグを変更しない。
        if *mode == AddressingMode::Immediate {
          return;
        }
        let flags = FLAG_NEGATIVE | FLAG_OVERFLOW;
        if self.is_accumulator_16bit_mode() {
          self.status = (self.status & !flags) | ((value >> 8) as u8 & flags);
        } else {
          self.status = (self.status & !flags) | (value as u8 & flags);
        }
    }

    pub fn bne(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_ZERO, false);
    }

    pub fn beq(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_ZERO, true);
    }

    pub fn bcc(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_CARRY, false);
    }

    pub fn bcs(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_CARRY, true);
    }

    pub fn ror(&mut self, mode: &AddressingMode) {
      let shift = if self.is_accumulator_16bit_mode() { 15 } else { 7 };
      let (value, carry) = if mode == &AddressingMode::Accumulator {
        let a = self.get_register_a();
        let carry = a & 0x01;
        let a = a >> 1;
        let a = a | ((self.status as u16 & FLAG_CARRY as u16) << shift);
        self.set_register_a(a);
        (a, carry)
      } else {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read_auto(addr);
        let carry = value & 0x01;
        let value = value >> 1;
        let value = value | ((self.status as u16 & FLAG_CARRY as u16) << shift);
        self.mem_write_auto(addr, value);
        (value, carry)
      };

      self.status = if carry != 0 {
        self.status | FLAG_CARRY
      } else {
        self.status & !FLAG_CARRY
      };
      self.update_zero_and_negative_flags(value);
    }

    pub fn rol(&mut self, mode: &AddressingMode) {
      let (value, carry) = if mode == &AddressingMode::Accumulator {
        let a = self.get_register_a();
        let (value, carry) = self.overflowing_mul(a, 2);
        self.set_register_a(value | (self.status & FLAG_CARRY) as u16);
        (self.get_register_a(), carry)
      } else {
          let addr = self.get_operand_address(mode);
          let value = self.mem_read_auto(addr);
          // println!("VALUE: {:04X}", value);
          let (value, carry) = self.overflowing_mul(value, 2);
          let value = value | (self.status & FLAG_CARRY) as u16;
          self.mem_write_auto(addr, value);
          (value, carry)
      };

      self.status = if carry {
          self.status | FLAG_CARRY
      } else {
          self.status & !FLAG_CARRY
      };
      self.update_zero_and_negative_flags(value);
    }

    pub fn lsr(&mut self, mode: &AddressingMode) {
      let (value, carry) = if mode == &AddressingMode::Accumulator {
          let a = self.get_register_a();
          let carry = a & 0x01;
          let a = a >> 1;
          self.set_register_a(a);
          (self.get_register_a(), carry)
      } else {
          let addr = self.get_operand_address(mode);
          let value = self.mem_read_auto(addr);
          let carry = value & 0x01;
          let value = value >> 1;
          self.mem_write_auto(addr, value);
          (value, carry)
      };

      self.status = if carry == 1 {
          self.status | FLAG_CARRY
      } else {
          self.status & !FLAG_CARRY
      };
      self.update_zero_and_negative_flags(value);
    }

    fn overflowing_mul(&self, lhs: u16, rhs: u16) -> (u16, bool) {
      if self.is_accumulator_16bit_mode() {
        lhs.overflowing_mul(rhs)
      } else {
        let (result, carry) = (lhs as u8).overflowing_mul(rhs as u8);
        (result as u16, carry)
      }
    }

    pub fn asl(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let a = self.get_register_a();
            let (value, carry) = self.overflowing_mul(a, 2);
            self.set_register_a(value);
            (value, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read_u16(addr);
            let (value, carry) = self.overflowing_mul(value, 2);
            self.mem_write_u16(addr, value); // TODO
            (value, carry)
        };

        self.status = if carry {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    pub fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value: u16 = self.mem_read_u16(addr);
        let a = self.get_register_a() | value;
        self.set_register_a(a);
        let a = self.get_register_a();
        self.update_zero_and_negative_flags(a);
    }

    pub fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read_u16(addr);
        let a = self.get_register_a() ^ value;
        self.set_register_a(a);
        let a = self.get_register_a();
        self.update_zero_and_negative_flags(a);
    }

    pub fn and(&mut self, mode: &AddressingMode) {
      let addr = self.get_operand_address(mode);
      let value = self.mem_read_u16(addr);
      let value = self.get_register_a() & value;
      self.set_register_a(value);
      self.update_zero_and_negative_flags(value);
    }

    pub fn sbc(&mut self, mode: &AddressingMode) {
      /*
        // A-M-(1-C)
        // キャリーかどうかの判定が逆
        // キャリーの引き算(1-C)
        // overflowの判定が逆 = m,p, p,m
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let carry = self.status & FLAG_CARRY;
        let (v1, carry_flag1) = self.register_a.overflowing_sub(value);
        let (n, carry_flag2) = v1.overflowing_sub(1 - carry);

        let overflow = (self.register_a & SIGN_BIT) != (value & SIGN_BIT)
            && (self.register_a & SIGN_BIT) != (n & SIGN_BIT);

        self.register_a = n;

        self.status = if !carry_flag1 && !carry_flag2 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.status = if overflow {
            self.status | FLAG_OVERFLOW
        } else {
            self.status & !FLAG_OVERFLOW
        };

        self.update_zero_and_negative_flags(self.register_a)
      */
      todo!("sbc");
    }

    fn overflowing_add(&self, lhs: u16, rhs: u16) -> (u16, bool) {
      if self.is_accumulator_16bit_mode() {
        lhs.overflowing_add(rhs)
      } else {
        let (result, carry) = (lhs as u8).overflowing_add(rhs as u8);
        (result as u16, carry)
      }
    }

    fn sign_bit(&self, value: u16) -> bool {
      if self.is_accumulator_16bit_mode() {
        (value & 0x8000) == 0
      } else {
        (value & 0x0080) == 0
      }
    }

/*
    fn _bcd2(&self, value: u16) -> u16 {
      if (self.status & FLAG_DECIMAL) == 0 {
        return value;
      }
      let n1 = ((value & 0x000F) >> 0) * 1;
      let n2 = ((value & 0x00F0) >> 4) * 10;
      let n3 = ((value & 0x0F00) >> 8) * 100;
      println!(" n1: {} n2: {} n3: {}", n1, n2, n3);
      let mut d = n1 + n2 + n3;
      d
    }

    fn _bcd3(&self, value: u16) -> u16 {
      if (self.status & FLAG_DECIMAL) == 0 {
        return value;
      }
      let mut d = value;
      let mut ret: u16 = 0;
      let mut shift = 0;
      while d > 0 {
        let m = d % 10;
        d = d / 10;
        ret += m << shift;
        shift += 4;
      }
      ret
    }
 */
    pub fn adc(&mut self, mode: &AddressingMode) {
        if (self.status & FLAG_DECIMAL) != 0 {
          // デシマルモード
          let addr = self.get_operand_address(mode);
          let value = self.mem_read_u16(addr);
          let a: u16 = self.get_register_a();
          let mut carry = (self.status & FLAG_CARRY) as u16;

          let mut result: u16 = 0;
          let mut value_l = value;
          let mut value_r = a;
          let mut overflow = false;
          for i in if self.is_accumulator_16bit_mode() { vec![0, 1, 2, 3] } else { vec![0, 1] } {
            let mut sum = (value_l & 0x000F) + (value_r & 0x000F) + carry;
            carry = 0;
            if sum > 9 {
              sum += 6;
              // overflow = sum >= 0x000F;
              sum &= 0x000F;
              carry = 1;
            }
            result = result | sum << (i * 4);
            value_l >>= 4;
            value_r >>= 4;
            overflow = (sum as i16) < -8 || (sum as i16) > 7;
          }
          println!("A: {:06X}, V: {:06X}, R: {:06X}", a, value, result);

          // let overflow = self.sign_bit(a) == self.sign_bit(value)
          //   && self.sign_bit(value) != self.sign_bit(result);

          self.set_register_a(result);
          let a = self.get_register_a();

          self.status = if carry != 0 {
              self.status | FLAG_CARRY
          } else {
              self.status & !FLAG_CARRY
          };
          self.status = if overflow {
              self.status | FLAG_OVERFLOW
          } else {
              self.status & !FLAG_OVERFLOW
          };

          self.update_zero_and_negative_flags(a);

          return;
        }
        // 通常時
        let addr = self.get_operand_address(mode);
        let value = self.mem_read_u16(addr);

        let carry = (self.status & FLAG_CARRY) as u16;
        let (rhs, carry_flag1) = self.overflowing_add(value, carry);
        let a = self.get_register_a();

        let (n, carry_flag2) = self.overflowing_add(a, rhs);
        let overflow = self.sign_bit(a) == self.sign_bit(value)
            && self.sign_bit(value) != self.sign_bit(n);

        self.set_register_a(n);
        let a = self.get_register_a();

        self.status = if carry_flag1 || carry_flag2 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.status = if overflow {
            self.status | FLAG_OVERFLOW
        } else {
            self.status & !FLAG_OVERFLOW
        };

        self.update_zero_and_negative_flags(a)
    }

    fn update_zero_and_negative_flags(&mut self, result: u16) {
        self._update_zero_and_negative_flags_by_register(result, true);
    }

    fn update_zero_and_negative_flags_xy(&mut self, result: u16) {
        self._update_zero_and_negative_flags_by_register(result, false);
    }

    fn _update_zero_and_negative_flags_by_register(&mut self, result: u16, mode_a: bool) {
        let mode = if mode_a {
          self.is_accumulator_16bit_mode()
        } else {
          self.is_index_register_16bit_mode()
        };
        self._update_zero_and_negative_flags(result, mode);
    }

    fn _update_zero_and_negative_flags(&mut self, result: u16, mode16bit: bool) {
        let test_bit = if mode16bit { 0x8000 } else { 0x0080 };
        self.status = if result == 0 {
            self.status | FLAG_ZERO
        } else {
            self.status & !FLAG_ZERO
        };
        self.status = if (result & test_bit) != 0 {
            self.status | FLAG_NEGATIVE
        } else {
            self.status & !FLAG_NEGATIVE
        }
    }
}

/*
pub fn trace(cpu: &mut CPU) -> String {
    // 0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD
    // OK 0064 => program_counter
    // OK A2 01 => binary code
    // OK LDX #$01 => asm code
    // "0400 @ 0400 = AA" => memory access
    // OK A:01 X:02 Y:03 P:24 SP:FD => register, status, stack_pointer
    unsafe { IN_TRACE = true };

    let program_counter = cpu.program_counter - 1;
    let pc = format!("{:<04X}", program_counter);
    let op = cpu.mem_read(program_counter as u32);
    let ops = CPU_OPS_CODES.get(&op).unwrap();
    let mut args: Vec<u8> = vec![];
    for n in 1..ops.bytes {
        let arg = cpu.mem_read(program_counter as u32 + n as u32);
        args.push(arg);
    }
    let bin = binary(op, &args);
    let asm = disasm(program_counter, &ops, &args);
    let memacc = memory_access(cpu, &ops, &args);
    let status = cpu2str(cpu);

    let log = format!(
        "{:<6}{:<9}{:<33}{}",
        pc,
        bin,
        vec![asm, memacc].join(" "),
        status
    );

    trace!("{}", log);

    unsafe { IN_TRACE = false };

    log
}

fn binary(op: u8, args: &Vec<u8>) -> String {
    let mut list: Vec<String> = vec![];
    list.push(format!("{:<02X}", op));
    for v in args {
        list.push(format!("{:<02X}", v));
    }
    list.join(" ")
}

fn disasm(program_counter: u16, ops: &OpCode, args: &Vec<u8>) -> String {
    let prefix = if ops.name.starts_with("*") { "" } else { " " };
    format!(
        "{}{} {}",
        prefix,
        ops.name,
        address(program_counter, &ops, args)
    )
}

fn address(program_counter: u16, ops: &OpCode, args: &Vec<u8>) -> String {
    match ops.addressing_mode {
        AddressingMode::Implied => {
            format!("")
        }
        AddressingMode::Accumulator => {
            format!("A")
        }
        // LDA #$44 => a9 44
        AddressingMode::Immediate => {
            format!("#${:<02X}", args[0])
        }

        // LDA $44 => a5 44
        AddressingMode::ZeroPage => {
            format!("${:<02X}", args[0])
        }

        // LDA $4400 => ad 00 44
        AddressingMode::Absolute => {
            format!("${:<02X}{:<02X}", args[1], args[0])
        }
        // LDA $44,X => b5 44
        AddressingMode::Direct_Page_Indexed_by_X => {
            format!("${:<02X},X", args[0])
        }

        // LDX $44,Y => b6 44
        AddressingMode::ZeroPage_Y => {
            format!("${:<02X},Y", args[0])
        }

        // LDA $4400,X => bd 00 44
        AddressingMode::Absolute_Indexed_by_X => {
            format!("${:<02X}{:<02X},X", args[1], args[0])
        }

        // LDA $4400,Y => b9 00 44
        AddressingMode::Absolute_Indexed_by_Y => {
            format!("${:<02X}{:<02X},Y", args[1], args[0])
        }
        // JMP
        AddressingMode::Indirect => {
            format!("(${:<02X}{:<02X})", args[1], args[0])
        }

        // LDA ($44,X) => a1 44
        AddressingMode::Indirect_X => {
            format!("(${:<02X},X)", args[0])
        }

        // LDA ($44),Y => b1 44
        AddressingMode::Indirect_Y => {
            format!("(${:<02X}),Y", args[0])
        }

        // BCC *+4 => 90 04
        AddressingMode::Relative => {
            format!(
                "${:<04X}",
                (program_counter as i32 + (args[0] as i8) as i32) as u16 + 2
            )
        }

        AddressingMode::NoneAddressing => {
            panic!("mode {:?} is not supported", ops.addressing_mode);
        }
    }
}

fn memory_access(cpu: &mut CPU, ops: &OpCode, args: &Vec<u8>) -> String {
    if ops.name.starts_with("J") {
        if ops.addressing_mode == AddressingMode::Indirect {
            let hi = args[1] as u16;
            let lo = args[0] as u16;
            let addr = hi << 8 | lo;
            let value = cpu.mem_read_u16(addr as u32);
            return format!("= {:<04X}", value);
        }
        return format!("");
    }

    match ops.addressing_mode {
        AddressingMode::ZeroPage => {
            let value = cpu.mem_read(args[0] as u32);
            format!("= {:<02X}", value)
        }
        AddressingMode::Direct_Page_Indexed_by_X => {
            let addr = (args[0] as u16).wrapping_add(cpu.register_x) as u16;
            let value = cpu.mem_read(addr as u32);
            format!("@ {:<02X} = {:<02X}", addr, value)
        }
        AddressingMode::ZeroPage_Y => {
            let addr = (args[0] as u16).wrapping_add(cpu.register_y) as u16;
            let value = cpu.mem_read(addr as u32);
            format!("@ {:<02X} = {:<02X}", addr, value)
        }
        AddressingMode::Absolute => {
            let hi = args[1] as u16;
            let lo = args[0] as u16;
            let addr = hi << 8 | lo;
            let value = cpu.mem_read(addr as u32);
            format!("= {:<02X}", value)
        }
        AddressingMode::Absolute_Indexed_by_X => {
            let hi = args[1] as u16;
            let lo = args[0] as u16;
            let base = hi << 8 | lo;
            let addr = base.wrapping_add(cpu.register_x as u16) as u32;
            let value = cpu.mem_read(addr);
            format!("@ {:<04X} = {:<02X}", addr, value)
        }
        AddressingMode::Absolute_Indexed_by_Y => {
            let hi = args[1] as u16;
            let lo = args[0] as u16;
            let base = hi << 8 | lo;
            let addr = base.wrapping_add(cpu.register_y as u16) as u32;
            let value = cpu.mem_read(addr);
            format!("@ {:<04X} = {:<02X}", addr, value)
        }
        AddressingMode::Indirect_X => {
            let base = args[0];
            // FIXME 元々の実装は、u8にすることで、上位8ビットを切っていた。
            let ptr: u16 = (base as u16).wrapping_add(cpu.register_x);
            let addr = cpu.mem_read_u16(ptr as u32) as u32;
            let value = cpu.mem_read(addr);
            format!("@ {:<02X} = {:<04X} = {:<02X}", ptr, addr, value)
        }
        AddressingMode::Indirect_Y => {
            let base = args[0];
            let deref_base = cpu.mem_read_u16(base as u32);
            let deref = deref_base.wrapping_add(cpu.register_y as u16) as u32;
            let value = cpu.mem_read(deref);
            format!("= {:<04X} @ {:<04X} = {:<02X}", deref_base, deref, value)
        }
        _ => {
            format!("")
        }
    }
}

fn cpu2str(cpu: &CPU) -> String {
    format!(
        "A:{:<02X} X:{:<02X} Y:{:<02X} P:{:<02X} SP:{:<02X}",
        cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.stack_pointer,
    )
}
*/
