use log::{debug, info, trace};

use crate::opscodes::{call, CPU_OPS_CODES};

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Accumulator,
    Immediate,
    ZeroPage_Y,
    Absolute,
    Absolute_Indexed_by_X,
    Absolute_Indexed_by_Y,
    Indirect,
    Indirect_X,
    Indirect_Y,
    Relative,
    Implied,

    // 65816
    Absolute_Long,
    Direct_Page, // ZeroPage
    Direct_Page_Indirect,
    Direct_Page_Indirect_Long,
    Absolute_Long_Indexed_by_X,
    Direct_Page_Indexed_by_X,
    Direct_Page_Indexed_by_Y,
    Direct_Page_Indexed_Indirect_by_X,
    Direct_Page_Indirect_Indexed_by_Y,
    Direct_Page_Indirect_Long_Indexed_by_Y,
    Stack_Relative,
    Stack_Relative_Indirect_Indexed_by_Y,

    NoneAddressing,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CycleCalcMode {
    None,
    Page,
    Branch,
}

#[derive(Debug, Clone)]
pub struct OpCode {
    pub code: u8,
    pub name: String,
    pub bytes: u16,
    pub cycles: u8,
    pub cycle_calc_mode: CycleCalcMode,
    pub addressing_mode: AddressingMode,
}

impl OpCode {
    pub fn new(
        code: u8,
        name: &str,
        bytes: u16,
        cycles: u8,
        cycle_calc_mode: CycleCalcMode,
        addressing_mode: AddressingMode,
    ) -> Self {
        OpCode {
            code: code,
            name: String::from(name),
            bytes: bytes,
            cycles: cycles,
            cycle_calc_mode: cycle_calc_mode,
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

const SIGN_BIT: u8 = 1 << 7; // FIXME FLAG_OVERFLOWで使用？

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

    add_cycles: u8,
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
            add_cycles: 0,
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

    fn is_accumulator_16bit_mode(&self) -> bool {
      self.is_native_mode() && (self.status & FLAG_MEMORY_ACCUMULATOR_MODE) == MODE_16BIT
    }


    fn is_index_register_16bit_mode(&self) -> bool {
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
      if self.is_accumulator_16bit_mode() {
        self.register_a
      } else {
        self.register_a & 0x00FF
      }
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
            AddressingMode::Implied => {
                panic!("AddressingMode::Implied");
            }
            AddressingMode::Accumulator => {
                panic!("AddressingMode::Accumulator");
            }
            // LDA #$44 => a9 44
            AddressingMode::Immediate => {
              pc
            },

            // LDA $44 => a5 44
            AddressingMode::Direct_Page => {
              // = Direct Pageなので、統合する必要あり。
              let addr = self.mem_read(pc) as u32;
              (self.direct_page as u32).wrapping_add(addr) & 0x00FFFF
            },

            // LDA $4400 => ad 00 44
            AddressingMode::Absolute => {
              let addr = self.mem_read_u16(pc) as u32;
              ((self.data_bank as u32) << 16) | addr
            }

            // LDA $44,X => b5 44
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

            // LDX $44,Y => b6 44
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(pc);
                let addr = pos.wrapping_add(self.register_y as u8) as u16;
                addr as u32
            }

            // LDA $4400,X => bd 00 44
            AddressingMode::Absolute_Indexed_by_X => {
                let base = self.mem_read_u16(pc);
                let addr = ((self.data_bank as u32) << 16) | base as u32;
                let addr = addr.wrapping_add(self.register_x as u32);
                addr & 0xFFFFFF
            }

            // LDA $4400,Y => b9 00 44
            AddressingMode::Absolute_Indexed_by_Y => {
                let base = self.mem_read_u16(pc);
                let addr = ((self.data_bank as u32) << 16) | base as u32;
                let addr = addr.wrapping_add(self.register_y as u32);
                addr & 0xFFFFFF
            }
            // JMP -> same Absolute
            AddressingMode::Indirect => {
                let base = self.mem_read_u16(pc);
                let addr = self.mem_read_u16(base as u32);
                addr as u32
            }

            // LDA ($44,X) => a1 44
            AddressingMode::Indirect_X => {
                let base = self.mem_read(pc);
                let ptr: u8 = (base as u8).wrapping_add(self.register_x as u8);
                let addr = self.mem_read_u16(ptr as u32);
                addr as u32
            }

            // LDA ($44),Y => b1 44
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(pc);
                let deref_base = self.mem_read_u16(base as u32);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                // (+1 if page crossed)
                if deref_base & 0xFF00 != deref & 0xFF00 {
                    self.add_cycles += 1;
                }
                deref as u32
            }
            // ↑ エミュレーションモード(8Bitモード)時のアドレッシングモード

            // ↓ ネイティブモード(16Bitモード)時のアドレッシングモード

            // BCC *+4 => 90 04
            AddressingMode::Relative => {
                let base = self.mem_read(pc);
                let np = (base as i8) as i32 + pc as i32;
                return np as u32;
            }

            // LDA long => AF 12 56 34
            AddressingMode::Absolute_Long => {
              let addr = self.mem_read_u16(pc);
              let bank = self.mem_read(pc + 2);
              (bank as u32) << 16 | addr as u32
            }
            // LDA ($12) => B2 12
            AddressingMode::Direct_Page_Indirect => {
              let addr = self.mem_read(pc) as u32;
              let addr = (self.direct_page as u32).wrapping_add(addr) & 0x00FFFF;
              ((self.data_bank as u32) << 16) | self.mem_read_u16(addr) as u32
            }
            // LDA [$12] => A7 12
            AddressingMode::Direct_Page_Indirect_Long => {
              let base = self.mem_read(pc) as u32;
              let base = (self.direct_page as u32).wrapping_add(base) & 0x00FFFF;
              let addr = self.mem_read_u16(base) as u32;
              let bank = self.mem_read((base + 2) & 0x00FFFF);
              ((bank as u32) << 16) | addr as u32
            }
            // TODO LDAには無いアドレッシングモード
            AddressingMode::Direct_Page_Indexed_by_Y => {
              let addr = self.mem_read(pc);
              let addr = self.direct_page.wrapping_add(addr as u16) as u32;
              addr.wrapping_add(self.register_y as u32)
            }
            // LDA (dp, X) => A1 dp
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
              let addr = self.mem_read_u16(addr) as u32;
              (self.data_bank as u32) << 16 | addr
            }
            // LDA (dp), Y => B1 dp
            AddressingMode::Direct_Page_Indirect_Indexed_by_Y => {
              // アドレス部の内容にダイレクトページレジスタの値を足して得られるアドレスから16bitを読み込み、さらにそれにYレジスタを足したものを下位16bit、DBレジスタを上位8bitとしたアドレスに目的のデータが格納されています。
              // ($12),yのように表します。
              let addr = self.mem_read(pc);
              let addr = (self.direct_page as u32).wrapping_add(addr as u32);
              let addr = addr & 0x00FFFF;
              let addr = self.mem_read_u16(addr) as u32;
              let addr = addr.wrapping_add(self.get_register_y() as u32);
              ((self.data_bank as u32) << 16).wrapping_add(addr) & 0xFFFFFF
            }
            AddressingMode::Absolute_Long_Indexed_by_X => {
                // アドレス部の内容にインデクスレジスタの値を足したアドレスが目的のデータが格納されている24bitフルアドレスを表します。
                // $123456,xと表します。絶対ロングアドレスインデクスYモードはありません。
                let base = self.mem_read_u16(pc);
                let bank = self.mem_read(pc + 2);
                let addr = ((bank as u32) << 16) | base as u32;
                let addr = addr.wrapping_add(self.register_x as u32);
                addr & 0xFFFFFF
            }
            AddressingMode::Direct_Page_Indirect_Long_Indexed_by_Y => {
              // アドレス部の内容にダイレクトページレジスタの値を足して得られるアドレスから24bitを読み込み、さらにYレジスタを足したアドレスに目的のデータが格納されています。
              // ダイレクトインダイレクトロングインデクスXモードはありません。[$12],yのように表します。
              let addr = self.mem_read(pc);
              let addr = (self.direct_page as u32).wrapping_add(addr as u32);
              let addr = addr & 0x00FFFF;
              let base = self.mem_read_u16(addr);
              let bank = self.mem_read(addr + 2);
              let addr = ((bank as u32) << 16) | base as u32;
              let addr = addr.wrapping_add(self.get_register_y() as u32);
              addr & 0xFFFFFF
            }
            AddressingMode::Stack_Relative => {
              // アドレス部の内容にスタックポインタを足したアドレスが目的のデータが格納されたアドレスを表します。
              // スタックポインタは常に次の有効なスタックの空き領域を示しているため、オペランドに1を指定すれば最後にスタックに積まれた値、0を指定すれば最後にスタックからプルされた値を指す。
              // $01,sのように表します。
              let value = self.mem_read(pc) as u32;
              // self.stack_pointer はnative modeかどうかを判断する必要あり。(16 / 8 bit切り替え)
              // 8bit modeだったら、上位バイトをクリアして、0x0100を足す。
              let addr = (self.stack_pointer as u32).wrapping_add(value);
              let addr = 0x0100 + addr;
              self.mem_read(addr) as u32
            }
            AddressingMode::Stack_Relative_Indirect_Indexed_by_Y => {
              todo!("Stack_Relative_Indirect_Indexed_by_Y")
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    pub fn mem_read_u16(&mut self, pos: u32) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    pub fn mem_write_u16(&mut self, pos: u32, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0x00FF) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
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

    fn apply_mode(&mut self) {
      if self.is_emulation_mode() {
        self.stack_pointer = 0x0100 | (self.stack_pointer & 0x00FF)
      }
    }

    pub fn run(&mut self) {
        self.apply_mode();
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
        match op {
            Some(op) => {
                self.add_cycles = 0;

                call(self, &op);

                match op.cycle_calc_mode {
                    CycleCalcMode::None => {
                        self.add_cycles = 0;
                    }
                    CycleCalcMode::Page => {
                        if self.add_cycles > 1 {
                            panic!(
                                "Unexpected cycle_calc. {} {:?} => {}",
                                op.name, op.addressing_mode, self.add_cycles
                            )
                        }
                    }
                    _ => {}
                }

                // self.bus.tick(op.cycles + self.add_cycles);

                // if program_conter_state == self.program_counter {
                //   self.program_counter += (op.len - 1) as u16
                // }
            }
            _ => {} // panic!("no implementation {:<02X}", opscode),
        }
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

/*
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
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let s = self._pop();
        self.register_a = value & s;
        self.register_x = self.register_a;
        self._push(self.register_a);
        self.update_zero_and_negative_flags(self.register_a);
        todo!("lae")
    }

    pub fn shx(&mut self, mode: &AddressingMode) {
        // M =3D X AND HIGH(arg) + 1
        let addr = self.get_operand_address(mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, (self.register_x & h).wrapping_add(1));
        todo!("shx")
    }

    pub fn shy(&mut self, mode: &AddressingMode) {
        // Y&H into {adr}
        // AND Y register with the high byte of the target address of the argument
        // + 1. Store the result in memory.
        let addr = self.get_operand_address(mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, (self.register_y & h).wrapping_add(1));
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
        self._push(self.register_a & self.register_x);
        let addr = self.get_operand_address(mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, self.register_a & self.register_x & h);
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
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a & self.register_x);
    }

    pub fn lax(&mut self, mode: &AddressingMode) {
        self.lda(mode);
        self.tax(mode);
    }

    pub fn txs(&mut self, mode: &AddressingMode) {
        self.stack_pointer = self.register_x;
    }

    pub fn tsx(&mut self, mode: &AddressingMode) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn tya(&mut self, mode: &AddressingMode) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn tay(&mut self, mode: &AddressingMode) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn txa(&mut self, mode: &AddressingMode) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn tax(&mut self, mode: &AddressingMode) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    pub fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    pub fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    pub fn rti(&mut self, mode: &AddressingMode) {
        // スタックからプロセッサ フラグをプルし、続いてプログラム カウンタをプルします。
        self.status = self._pop() & !FLAG_BREAK | FLAG_BREAK2;
        self.program_counter = self._pop_u16();
    }

    pub fn plp(&mut self, mode: &AddressingMode) {
        self.status = self._pop() & !FLAG_BREAK | FLAG_BREAK2;
    }

    pub fn php(&mut self, mode: &AddressingMode) {
        self._push(self.status | FLAG_BREAK | FLAG_BREAK2);
    }

    pub fn pla(&mut self, mode: &AddressingMode) {
        self.register_a = self._pop();
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn pha(&mut self, mode: &AddressingMode) {
        self._push(self.register_a);
    }

    pub fn nop(&mut self, mode: &AddressingMode) {
        // なにもしない
    }

    pub fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }
*/
    pub fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read_u16(addr);
        self.set_register_a(value);
        let a = self.get_register_a();
        self.update_zero_and_negative_flags(a);
    }
/*
    pub fn rts(&mut self, mode: &AddressingMode) {
        let value = self._pop_u16() + 1;
        self.program_counter = value;
    }

    pub fn jsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self._push_u16(self.program_counter + 2 - 1);
        self.program_counter = addr;
        // 後で+2するので整合性のため-2しておく
        self.program_counter -= 2;
    }

    pub fn _push(&mut self, value: u8) {
        let addr = 0x0100 + self.stack_pointer as u16;
        trace!("STACK PUSH: {:04X} => {:02X}", self.stack_pointer, value);
        self.mem_write(addr, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn _pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let addr = 0x0100 + self.stack_pointer as u16;
        trace!("STACK POP: {:02X}", self.stack_pointer);
        self.mem_read(addr)
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
        let addr = self.get_operand_address(mode);
        self.program_counter = addr;
        // 後で+2するので整合性のため-2しておく
        self.program_counter -= 2;
        // TODO
        // オリジナルの 6502 は、間接ベクトルがページ境界にある場合、ターゲット アドレスを正しくフェッチしません (たとえば、$xxFF で、xx は $00 から $FF までの任意の値です)。この場合、予想どおり $xxFF から LSB を取得しますが、$xx00 から MSB を取得します。これは、65SC02 などの最近のチップで修正されているため、互換性のために、間接ベクトルがページの最後にないことを常に確認してください。
    }

    pub fn iny(&mut self, mode: &AddressingMode) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn inx(&mut self, mode: &AddressingMode) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr).wrapping_add(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    pub fn dey(&mut self, mode: &AddressingMode) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn dex(&mut self, mode: &AddressingMode) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr).wrapping_sub(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    fn _cmp(&mut self, target: u8, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        if target >= value {
            self.sec(&AddressingMode::Implied);
        } else {
            self.clc(&AddressingMode::Implied);
        }
        let value = target.wrapping_sub(value);
        self.update_zero_and_negative_flags(value);
    }

    pub fn cpy(&mut self, mode: &AddressingMode) {
        self._cmp(self.register_y, mode);
    }

    pub fn cpx(&mut self, mode: &AddressingMode) {
        self._cmp(self.register_x, mode);
    }

    pub fn cmp(&mut self, mode: &AddressingMode) {
        self._cmp(self.register_a, mode);
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
                // (+1 if branch succeeds
                //  +2 if to a new page)
                //    => new pageの場合は、+1っぽい。
                //     https://pgate1.at-ninja.jp/NES_on_FPGA/nes_cpu.htm#clock
                self.add_cycles += 1;
                if (self.program_counter & 0xFF00) != (addr & 0xFF00) {
                    self.add_cycles += 1;
                }
                self.program_counter = addr
            }
        } else {
            if self.status & flag == 0 {
                // (+1 if branch succeeds
                //  +2 if to a new page)
                self.add_cycles += 1;
                if (self.program_counter & 0xFF00) != (addr & 0xFF00) {
                    self.add_cycles += 1;
                }
                self.program_counter = addr
            }
        }
    }

    pub fn brk(&mut self, mode: &AddressingMode) {
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
        let value = self.mem_read(addr);

        let zero = self.register_a & value;
        if zero == 0 {
            self.status = self.status | FLAG_ZERO;
        } else {
            self.status = self.status & !FLAG_ZERO;
        }
        let flags = FLAG_NEGATIVE | FLAG_OVERFLOW;
        self.status = (self.status & !flags) | (value & flags);
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
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let carry = self.register_a & 0x01;
            self.register_a = self.register_a / 2;
            self.register_a = self.register_a | ((self.status & FLAG_CARRY) << 7);
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry = value & 0x01;
            let value = value / 2;
            let value = value | ((self.status & FLAG_CARRY) << 7);
            self.mem_write(addr, value);
            (value, carry)
        };

        self.status = if carry == 1 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    pub fn rol(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let (value, carry) = self.register_a.overflowing_mul(2);
            self.register_a = value | (self.status & FLAG_CARRY);
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let (value, carry) = value.overflowing_mul(2);
            let value = value | (self.status & FLAG_CARRY);
            self.mem_write(addr, value);
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
            let carry = self.register_a & 0x01;
            self.register_a = self.register_a / 2;
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry = value & 0x01;
            let value = value / 2;
            self.mem_write(addr, value);
            (value, carry)
        };

        self.status = if carry == 1 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    pub fn asl(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let (value, carry) = self.register_a.overflowing_mul(2);
            self.register_a = value;
            (value, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let (value, carry) = value.overflowing_mul(2);
            self.mem_write(addr, value);
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
        let value = self.mem_read(addr);
        self.register_a = self.register_a | value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = self.register_a ^ value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = self.register_a & value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn sbc(&mut self, mode: &AddressingMode) {
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
    }

    pub fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let carry = self.status & FLAG_CARRY;
        let (rhs, carry_flag1) = value.overflowing_add(carry);
        let (n, carry_flag2) = self.register_a.overflowing_add(rhs);

        let overflow = (self.register_a & SIGN_BIT) == (value & SIGN_BIT)
            && (value & SIGN_BIT) != (n & SIGN_BIT);

        self.register_a = n;

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

        self.update_zero_and_negative_flags(self.register_a)
    }
*/
    fn update_zero_and_negative_flags(&mut self, result: u16) {
        self.status = if result == 0 {
            self.status | FLAG_ZERO
        } else {
            self.status & !FLAG_ZERO
        };

        let test_bit = if self.is_accumulator_16bit_mode() { 0x8000 } else { 0x0080 };
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
