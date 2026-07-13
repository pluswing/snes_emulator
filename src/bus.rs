use core::panic;

use crate::{cartridge::{self, Cartridge}, ppu::PPU};

pub struct Bus {
  wram: Vec<u8>,
  pub ppu: PPU,
  cartridge: Cartridge,

  // FIXME とりあえず
  pub memory: Vec<u8>, // size=0xFFFFFF

  // 4211h RO - TIMEUP  - H/VタイマーIRQフラグ
  timeup: u8,
  // 4212h RO - HVBJOY  - H/VBlankフラグ & 自動Joypadビジーフラグ (R)
  hvbjoy: u8,
  // 4213h RO - RDIO    - Joypad Programmable I/O Port (Input)
  rdio: u8,

  // DMA
  mdmean: u8, // 420Bh WO - MDMAEN  - GDMAチャネルレジスタ
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

  /*
  43x7h RW - DASBx   - Indirect HDMA Address (bank)                          (FFh)
  43x8h RW - A2AxL   - HDMA Table Current Address (low)                      (FFh)
  43x9h RW - A2AxH   - HDMA Table Current Address (high)                     (FFh)
  43xAh RW - NTRLx   - HDMA Line-Counter (from current Table entry)          (FFh)
  */
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
      cartridge,
      memory: vec![0; 0x100_0000],

      timeup: 0x00,
      hvbjoy: 0x00,
      rdio: 0x00,

      mdmean: 0x00,
      dmap: [0xFF; 8],
      bbad: [0xFF; 8],
      a1: [0x00FFFF; 8],
      das: [0xFFFFFF; 8],
    }
  }

  pub fn tick(&mut self, cycles: u8) {
    self.ppu.tick(cycles);
  }

  fn write_dma_registers(&mut self, addr: u16, data: u8) {
    match addr {
      0x420B => {
        // DMA有効
        self.mdmean = data;
        self.do_transfer();
      }
      _ => {
        // 43x0h RW - DMAPx   - DMA設定レジスタ
        // ~
        // 43xFh RW - MIRRx   - 43xBhのミラー (R/W)
        let register = addr & 0x000F;
        let channel = ((addr & 0x00F0) >> 4) as usize;
        match register {
          // 転送設定
          0 => self.dmap[channel] = data,
          // PPU アドレス (to)
          1 => self.bbad[channel] = data,
          // Memory アドレス (from)
          2 => self.a1[channel] = (self.a1[channel] & 0xFFFF00) | (data as u32),
          3 => self.a1[channel] = (self.a1[channel] & 0xFF00FF) | ((data as u32) << 8),
          4 => self.a1[channel] = (self.a1[channel] & 0x00FFFF) | ((data as u32) << 16),
          // 転送サイズ
          5 => self.das[channel] = (self.das[channel] & 0xFFFF00) | (data as u32),
          6 => self.das[channel] = (self.das[channel] & 0xFF00FF) | ((data as u32) << 8),
          7 => self.das[channel] = (self.das[channel] & 0x00FFFF) | ((data as u32) << 16),
          _ => panic!("not implemented write_dma_registers({:04X}, {:02X})", addr, data)
        }
      }
    }
  }

  fn do_transfer(&mut self) {
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

      match mode {
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
              println!("DMA({}) {:06X} {:04X}", i, memory_addr, ppu_addr);
              let v = self.mem_read(memory_addr as u32);
              self.mem_write(ppu_addr, v);
              memory_addr = (memory_addr & 0xFF0000) | ((memory_addr + (1 * increment_weight)) & 0x00FFFF);
            } else {
              panic!("not inplement DMA 2レジスタ1書き込み PPU to CPU")
            }
          }
        }
        // 0b000	1レジスタ1書き込み	1 バイト: p

        // 0b011	2レジスタ2書き込み(それぞれ)	4 バイト: p, p, p+1, p+1
        // 0b100	4レジスタ1書き込み	4 バイト: p, p+1, p+2, p+3
        // 0b101	2レジスタ2書き込み(交互)	4 バイト: p, p+1, p, p+1
        // 0b110	1レジスタ2書き込み	2 バイト: p, p
        // 0b111	2レジスタ2書き込み(それぞれ)	4 バイト: p, p, p+1, p+1
        _ => panic!("not inplement DMA mode ({:03b})", mode)
      }
    }
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
    match bank {
      0x00..=0x3F => {
        match addr {
          0x0000..=0x1FFF => self.wram[addr as usize],
          0x2100..=0x213F => self.ppu.read(addr),
          0x4210 => {
            // 4210h RO - RDNMI   - NMIフラグ (Read/Ack)
            self.ppu.read(addr)
          }
          0x4211 => {
            let v = self.timeup;
            self.timeup = 0x00;
            v
          }
          0x4212 => self.hvbjoy,
          0x4213 => self.rdio,
          0x4214..=0x421F => {
            println!("mem_read({:02X}:{:04X})", bank, addr);
            0
          }
          // 4210h RO - RDNMI   - NMIフラグ (Read/Ack)
          // ~
          // 421Fh RO - JOY4H   - Joypad4レジスタ (上位8bit)
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
    match bank {
    0x00..=0x3F => {
        match addr {
          0x0000..=0x1FFF => self.wram[addr as usize] = data,
          0x2100..=0x213F => self.ppu.write(addr, data),
          0x420B => {
            self.write_dma_registers(addr, data);
          },
          0x4200..=0x420D => {
            // 4200h WO - NMITIMEN- 割り込み有効化レジスタ
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
