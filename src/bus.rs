use core::panic;

use sdl3::libc::PROC_PIDTBSDINFO;

use crate::{cartridge::{self, Cartridge}, ppu::PPU};

pub struct Bus {
  wram: Vec<u8>,
  pub ppu: PPU,
  cartridge: Cartridge,

  // FIXME とりあえず
  pub memory: Vec<u8>, // size=0xFFFFFF

  // DMA
  mdmean: u8, // 420Bh WO - MDMAEN  - GDMAチャネルレジスタ
  dmap0: u8, // 43x0h RW - DMAPx   - DMA設定レジスタ
  bbad0: u8, // 43x1h RW - BBADx   - DBバスアドレス
  a1t0l: u8, // 43x2h RW - A1TxL   - Aバスアドレス (low)
  a1t0h: u8, // 43x3h RW - A1TxH   - Aバスアドレス (high)
  a1b0: u8, // 43x4h RW - A1Bx    - Aバスアドレス (bank)
  das0l: u8, // 43x5h RW - DASxL   - Indirect HDMA Address (low)  / DMA Byte-Counter (low)
  das0h: u8, // 43x6h RW - DASxH   - Indirect HDMA Address (high) / DMA Byte-Counter (high)

  /*
  43x7h RW - DASBx   - Indirect HDMA Address (bank)                          (FFh)
  43x8h RW - A2AxL   - HDMA Table Current Address (low)                      (FFh)
  43x9h RW - A2AxH   - HDMA Table Current Address (high)                     (FFh)
  43xAh RW - NTRLx   - HDMA Line-Counter (from current Table entry)          (FFh)
  */
}

impl Bus {
  pub fn new(ppu: PPU, cartridge: Cartridge) -> Self {
    Self {
      wram: vec![0; 0x1_0000 * 2],
      ppu,
      cartridge,
      memory: vec![0; 0x100_0000],

      mdmean: 0x00,
      dmap0: 0xFF,
      bbad0: 0xFF,
      a1t0l: 0xFF,
      a1t0h: 0xFF,
      a1b0: 0x00,
      das0l: 0xFF,
      das0h: 0xFF,
    }
  }

  pub fn tick(&mut self, cycles: u8) {
    self.ppu.tick(cycles);
  }

  fn write_dma_registers(&mut self, addr: u16, data: u8) {
    // 43x0h RW - DMAPx   - DMA設定レジスタ
    // ~
    // 43xFh RW - MIRRx   - 43xBhのミラー (R/W)
    match addr {
      0x420B => {
        // DMA有効
        self.mdmean = data;
        self.do_transfer();
      }
      0x4300 => {
        // 転送設定
        self.dmap0 = data; // 0x09 = 00001001
      }
      // PPU アドレス (to)
      0x4301 => self.bbad0 = data,
      // Memory アドレス (from)
      0x4302 => self.a1t0l = data,
      0x4303 => self.a1t0h = data,
      0x4304 => self.a1b0 = data,
      // 転送サイズ
      0x4305 => self.das0l = data,
      0x4306 => self.das0h = data,
      _ => panic!("not implemented write_dma_registers({:04X}, {:02X})", addr, data)
    }
  }

  fn do_transfer(&mut self) {
    if self.mdmean & 0x01 != 0 {
      // DMAチャネル0が有効 => 転送する！

      // CPUメモリから読み込み、PPUレジスタ
      // 1バイトごとにDMAアドレスがインクリメント
      // DMAアドレスは固定されない
      // 2レジスタ1書き込み	2 バイト: p, p+1
      let mut memory_addr = (self.a1b0 as u32) << 16 | (self.a1t0h as u32) << 8 | (self.a1t0l as u32);
      let ppu_addr = 0x002100 | (self.bbad0 as u32); // $00:2100 ～ $00:21ff
      let transfer_size = (self.das0h as u32) << 8 | (self.das0l as u32);
      let transfer_size = if transfer_size == 0 { 0x10000 } else { transfer_size };

      // do_transfer M: 00CD35, P: 002118, S: 10000
      for i in 0..transfer_size {
        let v = self.mem_read(memory_addr);
        self.mem_write(if (i % 2) == 0 { ppu_addr } else { ppu_addr + 1 }, v);
        memory_addr = (memory_addr & 0xFF0000) | ((memory_addr + 1) & 0x00FFFF);
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
            // 4211h RO - TIMEUP  - H/VタイマーIRQフラグ
            0x80 // TODO
          }
          0x4212..=0x421F => {
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
