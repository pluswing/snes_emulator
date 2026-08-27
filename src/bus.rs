use core::panic;

use crate::{cartridge::{self, Cartridge}, ppu::PPU, apu::APU};

#[repr(u8)]
enum MemorySpeed {
  Fast = 6,
  Slow = 8,
  XSlow = 12
}

fn memory_speed(bank: u8, addr: u16) -> MemorySpeed {
  match bank {
    0x00..=0x3F => {
      match addr {
        0x0000..=0x1FFF => MemorySpeed::Slow,
        0x2000..=0x3FFF => MemorySpeed::Fast,
        0x4000..=0x41FF => MemorySpeed::XSlow,
        0x4200..=0x5FFF => MemorySpeed::Fast,
        0x6000..=0xFFFF => MemorySpeed::Slow,
      }
    }
    0x40..=0x7F => {
      MemorySpeed::Slow
    }
    0x80..=0xBF => {
      match addr {
        0x0000..=0x1FFF => MemorySpeed::Slow,
        0x2000..=0x3FFF => MemorySpeed::Fast,
        0x4000..=0x41FF => MemorySpeed::XSlow,
        0x4200..=0x5FFF => MemorySpeed::Fast,
        0x6000..=0x7FFF => MemorySpeed::Slow,
        // 注2 CPU レジスタ 0x420D のビット 0 がセットされている時、 スピードは Fast になり、セットされていない場合は Slow になる。
        0x8000..=0xFFFF => MemorySpeed::Slow // FIXME 注2
      }
    }
    0xC0..=0xFF => MemorySpeed::Slow // FIXME 注2
  }
}


pub struct Bus {
  wram: Vec<u8>,
  pub ppu: PPU,
  pub apu: APU,
  cartridge: Cartridge,
  pub cycles: u32,

  // FIXME とりあえず
  pub memory: Vec<u8>, // size=0xFFFFFF

  // 4213h RO - RDIO    - Joypad Programmable I/O Port (Input)
  rdio: u8,

  // DMA / HDMA
  mdmean: u8, // 420Bh WO - MDMAEN  - GDMAチャネルレジスタ
  hdmean: u8, // 420Ch WO - HDMAEN  - HDMAチャネルレジスタ
  dmap: [u8; 8], // 43x0h RW - DMAPx   - DMA設定レジスタ
  bbad: [u8; 8], // 43x1h RW - BBADx   - DBバスアドレス
  // 43x2h RW - A1TxL   - Aバスアドレス (low)
  // 43x3h RW - A1TxH   - Aバスアドレス (high)
  // 43x4h RW - A1Bx    - Aバスアドレス (bank)
  a1: [u32; 8],
  // 43x5h RW - DASxL   - Indirect HDMA Address (low)  / DMA Byte-Counter (low)
  // 43x6h RW - DASxH   - Indirect HDMA Address (high) / DMA Byte-Counter (high)
  // 43x7h RW - DASBx   - Indirect HDMA Address (bank)
  das: [u32; 8],
  // 43x8h RW - A2AxL   - HDMA Table Current Address (low)
  // 43x9h RW - A2AxH   - HDMA Table Current Address (high)
  a2: [u16; 8],
  // 43xAh RW - NTRLx   - HDMA Line-Counter (from current Table entry)
  ntrl: [u8; 8],

  // wram
  // 2181h WO - WMADDL  - WRAMアドレスレジスタ (下位8bit)  (W)
  // 2182h WO - WMADDM  - WRAMアドレスレジスタ (中位8bit) (W)
  // 2183h WO - WMADDH  - WRAMアドレスレジスタ (上位1bit)  (W)
  wmadd: u32,

  transfered_hdma: bool,
  transfer: [bool; 8],
  hdma_transfer_finished: [bool; 8],
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
enum DMADrection {
  CPU_TO_PPU,
  PPU_TO_CPU,
}

impl Bus {
  pub fn new(ppu: PPU, cartridge: Cartridge) -> Self {
    Self {
      wram: vec![0; 0x1_0000 * 2],
      ppu,
      apu: APU::new(),
      cartridge,
      cycles: 0,
      memory: vec![0; 0x100_0000],

      rdio: 0x00,

      mdmean: 0x00,
      hdmean: 0x00,
      dmap: [0xFF; 8],
      bbad: [0xFF; 8],
      a1: [0x00FFFF; 8],
      das: [0xFFFFFF; 8],
      a2: [0xFFFF; 8],
      ntrl: [0xFF; 8],

      wmadd: 0x000000,

      transfered_hdma: false,
      transfer: [false; 8],
      hdma_transfer_finished: [false; 8],
    }
  }

  pub fn tick(&mut self) {
    self.ppu.tick(self.cycles);
    self.hdma_transfer();
    self.cycles = 0;
  }

  fn write_dma_registers(&mut self, addr: u16, data: u8) {
    match addr {
      0x420B => {
        // DMA有効
        self.mdmean = data;
        self.dma_transfer();
      }
      0x420C => {
        // HDMA有効
        self.hdmean = data;
      }
      _ => {
        // 43x0h RW - DMAPx   - DMA設定レジスタ
        // ~
        // 43xFh RW - MIRRx   - 43xBhのミラー (R/W)
        let register = addr & 0x000F;
        let channel = ((addr & 0x00F0) >> 4) as usize;
        match register {
          // 転送設定
          0x00 => self.dmap[channel] = data,
          // PPU アドレス (to)
          0x01 => self.bbad[channel] = data,
          // Memory アドレス (from)
          0x02 => self.a1[channel] = (self.a1[channel] & 0xFFFF00) | (data as u32),
          0x03 => self.a1[channel] = (self.a1[channel] & 0xFF00FF) | ((data as u32) << 8),
          0x04 => self.a1[channel] = (self.a1[channel] & 0x00FFFF) | ((data as u32) << 16),
          // 転送サイズ
          0x05 => self.das[channel] = (self.das[channel] & 0xFFFF00) | (data as u32),
          0x06 => self.das[channel] = (self.das[channel] & 0xFF00FF) | ((data as u32) << 8),
          0x07 => self.das[channel] = (self.das[channel] & 0x00FFFF) | ((data as u32) << 16),
          0x08 => self.a2[channel] = (self.a2[channel] & 0xFF00) | (data as u16),
          0x09 => self.a2[channel] = (self.a2[channel] & 0x00FF) | (data as u16) << 8,
          0x0A => self.ntrl[channel] = data,
          _ => panic!("not implemented write_dma_registers({:04X}, {:02X})", addr, data)
        }
      }
    }
  }

  fn dma_transfer(&mut self) {
    for channel in 0..=7 {
      if self.mdmean & (0x01 << channel) == 0 {
        continue;
      }
      let channel = channel as usize;

      let mut memory_addr = self.a1[channel] as i64;
      let ppu_addr = 0x002100 | (self.bbad[channel] as u32);
      let transfer_size = self.das[channel] & 0x00FFFF;
      let transfer_size = if transfer_size == 0 { 0x10000 } else { transfer_size };

      let increment_direction: i64 = if (self.dmap[channel] & 0x10) == 0 { 1 } else { -1 };
      let increment_weight: i64 = if (self.dmap[channel] & 0x08) == 0 { increment_direction } else { 0 };
      let direction = if (self.dmap[channel] & 0x80) == 0 {
        DMADrection::CPU_TO_PPU
      } else {
        DMADrection::PPU_TO_CPU
      };
      let mode = self.dmap[channel] & 0x07;
      println!("DMA({}) {:06X} :: {:04X} S:{:04X}", channel, memory_addr, ppu_addr, transfer_size);

      match mode {
        0b000 => {
          // 1レジスタ1書き込み	1 バイト: p
          for _ in 0..transfer_size {
            if direction == DMADrection::CPU_TO_PPU {
              let v = self.mem_read(memory_addr as u32);
              self.mem_write(ppu_addr, v);
              memory_addr = (memory_addr & 0xFF0000) | ((memory_addr + (1 * increment_weight)) & 0x00FFFF);
            } else {
              panic!("not inplement DMA 1レジスタ1書き込み PPU to CPU")
            }
          }
        }
        0b001	=> {
          // 2レジスタ1書き込み
          for i in 0..transfer_size {
            if direction == DMADrection::CPU_TO_PPU {
              let v = self.mem_read(memory_addr as u32);
              self.mem_write(if (i % 2) == 0 { ppu_addr } else { ppu_addr + 1 }, v);
              memory_addr = (memory_addr & 0xFF0000) | ((memory_addr + (1 * increment_weight)) & 0x00FFFF);
            } else {
              panic!("not inplement DMA 2レジスタ1書き込み PPU to CPU")
            }
          }
        }
        0b010 => {
          // panic!("DMA MODE2 {:02X} {:04X}", self.dmap[channel], transfer_size);
          // 0b010	1レジスタ2書き込み	2 バイト: p, p
          for i in 0..transfer_size {
            if direction == DMADrection::CPU_TO_PPU {
              let v = self.mem_read(memory_addr as u32);
              self.mem_write(ppu_addr, v);
              // TODO あってる？
              let v = self.mem_read(memory_addr as u32 + 1);
              self.mem_write(ppu_addr, v);
              memory_addr = (memory_addr & 0xFF0000) | ((memory_addr + (1 * increment_weight)) & 0x00FFFF);
            } else {
              panic!("not inplement DMA 2レジスタ1書き込み PPU to CPU")
            }
          }
        }

        // 0b011	2レジスタ2書き込み(それぞれ)	4 バイト: p, p, p+1, p+1
        // 0b100	4レジスタ1書き込み	4 バイト: p, p+1, p+2, p+3
        // 0b101	2レジスタ2書き込み(交互)	4 バイト: p, p+1, p, p+1
        // 0b110	1レジスタ2書き込み	2 バイト: p, p
        // 0b111	2レジスタ2書き込み(それぞれ)	4 バイト: p, p, p+1, p+1
        _ => panic!("not inplement DMA mode ({:03b})", mode)
      }
    }
  }

  fn hdma_transfer(&mut self) {
    if self.hdmean == 0 {
      return
    }
    // Vブランク中はHDMAしない
    if self.ppu.vblank_flag {
      self.hdma_transfer_finished = [false; 8];
      return
    }
    if !self.ppu.hblank_flag {
      self.transfered_hdma = false;
      return
    }
    if self.transfered_hdma {
      return
    }
    self.transfered_hdma = true;

    for channel in 0..=7 {
      if self.hdmean & (0x01 << channel) == 0 {
        continue;
      }

      if self.hdma_transfer_finished[channel] {
        continue;
      }

      let channel = channel as usize;
      let addressing_mode = self.dmap[channel] & 0x40 == 0; // true=直接

      if self.ppu.v_counter == 0 {
        // 「アドレス」値に、Aアドレスがコピーされる
        // テーブルから 0x43xA に対して値をロードする (0x00 をロードすると、その場でチャネルを停止する…だろう)
        // 必要なら、間接アドレスをロードする => addressing_mode
        // 転送実行フラグ(DoTransfer) をTrueにする
        self.a2[channel] = (self.a1[channel] & 0x00FFFF) as u16;
        self.ntrl[channel] = self.mem_read((self.a1[channel] & 0xFF0000) | self.a2[channel] as u32);
        self.a2[channel] = self.a2[channel].wrapping_add(1);
        if addressing_mode == false {
          // 間接
          let addr_l = self.mem_read((self.a1[channel] & 0xFF0000) | self.a2[channel] as u32) as u32;
          self.a2[channel] = self.a2[channel].wrapping_add(1);
          let addr_h = self.mem_read((self.a1[channel] & 0xFF0000) | self.a2[channel] as u32) as u32;
          self.a2[channel] = self.a2[channel].wrapping_add(1);
          self.das[channel] = (self.das[channel] & 0xFF0000) | (addr_h << 8) | addr_l;
        }
        self.transfer[channel] = true;
      }

      let mode = self.dmap[channel] & 0x07;
      let direction = if (self.dmap[channel] & 0x80) == 0 {
        DMADrection::CPU_TO_PPU
      } else {
        DMADrection::PPU_TO_CPU
      };
      let ppu_addr = 0x002100 | (self.bbad[channel] as u32);

      if self.transfer[channel] {
        // アドレス/間接アドレスのいずれかから1バイト読み込み、インクリメントする。
        // -> modeによって、2バイトよむとかも必要。なはず。
        let value = if addressing_mode {
          // 直接
          let v = self.mem_read((self.a1[channel] & 0xFF0000) | self.a2[channel] as u32);
          self.a2[channel] = self.a2[channel].wrapping_add(1);
          v
        } else {
          // 間接
          let addr_l = self.mem_read(self.das[channel]) as u32;
          self.das[channel] = self.das[channel].wrapping_add(1);
          let addr_h = self.mem_read(self.das[channel]) as u32;
          self.das[channel] = self.das[channel].wrapping_add(1);
          let v = self.mem_read((addr_h << 8) | addr_l);
          v
        };

        // ポートに1バイト書き込む。Port+1, Port+2, Port+3 も、転送モードによっては書き込む。
        // 書き込みもモードのよって、2バイトとかもある。
        match mode {
          0b000	=> {
            // 1レジスタ1書き込み	1 バイト: p
            if direction == DMADrection::CPU_TO_PPU {
              self.mem_write(ppu_addr, value);
            } else {
              panic!("not inplement HDMA 1レジスタ1書き込み PPU to CPU")
            }
          }
          // 0b001	=> {
          // }
          0b010 => {
            // 1レジスタ2書き込み	2 バイト: p, p
            if direction == DMADrection::CPU_TO_PPU {
              self.mem_write(ppu_addr, value);
              self.mem_write(ppu_addr + 1, value);
            } else {
              panic!("not inplement HDMA 1レジスタ2書き込み PPU to CPU")
            }
          }

          // 0b011	2レジスタ2書き込み(それぞれ)	4 バイト: p, p, p+1, p+1
          // 0b100	4レジスタ1書き込み	4 バイト: p, p+1, p+2, p+3
          // 0b101	2レジスタ2書き込み(交互)	4 バイト: p, p+1, p, p+1
          // 0b110	1レジスタ2書き込み	2 バイト: p, p
          // 0b111	2レジスタ2書き込み(それぞれ)	4 バイト: p, p, p+1, p+1
          _ => panic!("not inplement HDMA mode ({:03b})", mode)
        }
      }

      // 3. 0x43xA をデクリメント
      // 転送実行フラグ(DoTransfer) に「繰り返し」ビットと同じ値をセット
      self.ntrl[channel] = self.ntrl[channel].wrapping_sub(1);
      self.transfer[channel] = self.ntrl[channel] & 0x7F != 0;

      let line_counter = self.ntrl[channel] & 0x7F;
      if line_counter == 0 {
        // 「アドレス」から次の1バイトを読み込み、0x43xA に入れる(行番号と繰り返しも同じようにする)。
        // 間接アドレスモードの時、「アドレス」から2バイト読み込み、「間接アドレス」に入れ、 「アドレス」の値を2バイト分インクリメントする。
        // 注(奇妙な動作)：0x43xA が 0 で、処理中のチャネルが現在の行で最後のHDMAチャネルだった場合、 「アドレス」から1バイトのみ読み込まれ、下位バイトには0x00が適用される。 「アドレス」は1つだけインクリメントされ、1少ないCPUサイクルが消費される。
        // 0x43xA が0の時、処理中のHDMAチャネルのこのフレームでの転送は終了する。 0x420c のビットはクリアされないが、次のフレームには自動的に開始される。
        // 転送実行フラグ(DoTransfer) をTrueにする
        self.ntrl[channel] = self.mem_read((self.a1[channel] & 0xFF0000) | self.a2[channel] as u32);
        self.a2[channel] = self.a2[channel].wrapping_add(1);
        if addressing_mode == false {
          // 間接
          let addr_l = self.mem_read((self.a1[channel] & 0xFF0000) | self.a2[channel] as u32) as u32;
          self.a2[channel] = self.a2[channel].wrapping_add(1);
          let addr_h = self.mem_read((self.a1[channel] & 0xFF0000) | self.a2[channel] as u32) as u32;
          self.a2[channel] = self.a2[channel].wrapping_add(1);
          self.das[channel] = (self.das[channel] & 0xFF0000) | (addr_h << 8) | addr_l;
        }
        if self.ntrl[channel] == 0 {
          // 転送終了
          self.hdma_transfer_finished[channel] = true;
        }
        self.transfer[channel] = true;
      }
    }
  }

  fn write_wram_registers(&mut self, addr: u16, data: u8) {
    match addr {
      0x2180 => self.write_wram(data),
      0x2181 => self.wmadd = (self.wmadd & 0xFFFF00) | (data as u32),
      0x2182 => self.wmadd = (self.wmadd & 0xFF00FF) | ((data as u32) << 8),
      0x2183 => self.wmadd = (self.wmadd & 0x00FFFF) | ((data as u32) << 16),
      _ => panic!("not implemented write_wram_registers({:04X})", addr),
    }
  }

  fn read_wram_registers(&mut self, addr: u16) -> u8 {
    match addr {
      0x2180 => self.read_wram(),
      0x2181 => 0, // TODO open bus
      0x2182 => 0, // TODO open bus
      0x2183 => 0, // TODO open bus
      _ => panic!("not implemented read_wram_registers({:04X})", addr),
    }
  }

  fn write_wram(&mut self, data: u8) {
    // println!("write_wram({:06X}, {:02X})", self.wmadd, data);
    // FIXME: DMA で、WRAM からこのレジスタにアクセスすることはできず、 WRAM への書き込み操作は実行されません。
    // self.mem_write(self.wmadd, data);
    self.wmadd = self.wmadd.wrapping_add(1);
  }

  fn read_wram(&mut self) -> u8 {
    let v = self.mem_read(self.wmadd);
    self.wmadd = self.wmadd.wrapping_add(1);
    v
  }
}


pub trait Mem {
  fn mem_read(&mut self, addr: u32) -> u8;
  fn mem_write(&mut self, addr: u32, data: u8);
}

// [0x000000 ~ 0xFFFFFF]
// 0x0000~0xFFFF => 64KB * 2

impl Mem for Bus {
  fn mem_read(&mut self, addr: u32) -> u8 {
    let bank = ((addr & 0xFF_0000) >> 16) as u8;
    let addr = (addr & 0x00_FFFF) as u16;

    self.cycles += memory_speed(bank, addr) as u32;

    match bank {
      0x00..=0x3F => {
        match addr {
          0x0000..=0x1FFF => {
            // println!("READ WRAM {:04X} => {:02X}", addr, self.wram[addr as usize]);
            self.wram[addr as usize]
          }
          0x2100..=0x213F => self.ppu.read(addr),
          0x2140..=0x217F => self.apu.read(addr),
          0x2180..=0x2183 => self.read_wram_registers(addr),
          0x4210..=0x4212 => self.ppu.read(addr),
          0x4213 => self.rdio,
          0x4214..=0x421F => {
            println!("mem_read({:02X}:{:04X})", bank, addr);
            0
          }
          0x454C | 0x5241 | 0x5242 => 0, // TODO マリオコレクションでアクセス
          0x8000..=0xFFFF => self.cartridge.read(bank, addr),
          _ => panic!("not implemented mem_read({:02X}:{:04X})", bank, addr)
        }
      }
      0x40..=0x7D => {
        self.cartridge.read(bank, addr)
      }
      0x7E..=0x7F => {
        self.wram[addr as usize]
      }
      0x80..=0xBF => {
        match addr {
          0x0000..=0x1FFF => self.wram[addr as usize],
          0x2100..=0x213F => self.ppu.read(addr),
          // 0x4210..=0x4212 => self.ppu.read(addr),
          0x8000..=0xFFFF => self.cartridge.read(bank, addr),
          _ => panic!("not implemented mem_read({:02X}:{:04X})", bank, addr)
        }
      }
      0xC0..=0xFF => {
        self.cartridge.read(bank, addr)
      }
      _ => panic!("not implemented mem_read({:02X}:{:04X})", bank, addr)
    }
  }

  fn mem_write(&mut self, addr: u32, data: u8) {
    let bank = ((addr & 0xFF_0000) >> 16) as u8;
    let addr = (addr & 0x00_FFFF) as u16;

    self.cycles += memory_speed(bank, addr) as u32;

    match bank {
    0x00..=0x3F => {
        match addr {
          0x0000..=0x1FFF => {
            // println!("WRITE WRAM {:04X} => {:02X}", addr, data);
            self.wram[addr as usize] = data;
          }
          0x2100..=0x213F => self.ppu.write(addr, data),
          0x2140..=0x217F => self.apu.write(addr, data),
          0x2180..=0x2183 => self.write_wram_registers(addr, data),
          0x2184 => {}, // TODO マリオコレクションでアクセス（これなに？）
          0x4200..=0x4201 => self.ppu.write(addr, data),
          0x420B => self.write_dma_registers(addr, data),
          0x420C => self.write_dma_registers(addr, data),
          0x4202..=0x420D => {
            // ~
            // 420Dh WO - MEMSEL  - WS2制御レジスタ
            println!("mem_write({:02X}:{:04X}, {:02X})", bank, addr, data)
          }
          0x4300..=0x437F => {
            self.write_dma_registers(addr, data);
          }
          // 0x8000..=0xFFFF => self.cartridge.read(bank, addr),
          _ => panic!("not implemented mem_write({:02X}:{:04X}, {:02X})", bank, addr, data)
        }
      }
      0x40..=0x7D => {
        // self.cartridge.read(bank, addr)
      }
      0x7E..=0x7F => {
        self.wram[addr as usize] = data
      }
      0x80..=0xBF => {
        match addr {
          0x0000..=0x1FFF => self.wram[addr as usize] = data,
          0x2100..=0x213F => self.ppu.write(addr, data),
          // 0x8000..=0xFFFF => self.cartridge.read(bank, addr),
          _ => panic!("not implemented mem_write({:02X}:{:04X}, {:02X})", bank, addr, data)
        }
      }
      0xC0..=0xFF => {
        // self.cartridge.read(bank, addr)
      }
      _ => panic!("not implemented mem_write({:02X}:{:04X}, {:02X})", bank, addr, data)
    }
  }
}
