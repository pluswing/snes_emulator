use crate::{cartridge::{self, Cartridge}, ppu::PPU};

pub struct Bus {
  wram: Vec<u8>,
  pub ppu: PPU,
  cartridge: Cartridge,

  // FIXME とりあえず
  pub memory: Vec<u8>, // size=0xFFFFFF

  // DMA
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
      0x4300 => {
        self.dmap0 = data;
        self.do_transfer();
      }
      _ => panic!("not implemented write_dma_registers({:04X}, {:02X})", addr, data)
    }
  }

  fn do_transfer(&mut self) {
    // TODO
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
            // 420Bh WO - MDMAEN  - GDMAチャネルレジスタ 0
            // panic!("MDMAEN");
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
