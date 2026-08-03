use core::panic;
use std::ops::{Range, RangeInclusive};

fn bgr555_to_rgb888(data: u16) -> [u8; 3] {
  // .BBB BBGG GGGR RRRR
  let r = (data & 0x001F) as u8;
  let g = ((data >> 5) & 0x001F) as u8;
  let b = ((data >> 10) & 0x001F) as u8;
  let r = r << 3 | (r >> 2 & 0x07);
  let g = g << 3 | (g >> 2 & 0x07);
  let b = b << 3 | (b >> 2 & 0x07);
  [r, g, b]
}

pub struct PPU {
  cycles: u32,
  // registers
  pub inidisp: u8, // 2100h WO - INIDISP - ディスプレイ制御レジスタ1
  pub obsel: u8, // 2101h WO - OBSEL   - Object Size and Object Base
  pub oamaddl: u8, // 2102h WO - OAMADDL - OAMアドレス (下位8bit)
  pub oamaddh: u8,// 2103h WO - OAMADDH - OAMアドレス (上位1bit)
  pub oamdata: u8, // 2104h WO - OAMDATA - OAM書き込み
  pub bgmode: u8, // 2105h WO - BGMODE  - BG制御レジスタ
  pub mosaic: u8, // 2106h WO - MOSAIC  - モザイク
  pub bg1sc: u8, // 2107h WO - BG1SC   - BG1画面設定
  pub bg12nba: u8, // 210Bh WO - BG12NBA - BG1,2タイルデータアドレス
  // 2byteあるっぽい
  // BGnHOFS の場合 : (NewByte<<8) | (PrevByte&~7) | ((CurrentValue>>8)&7)
  // BGnVOFS の場合 : (NewByte<<8) | PrevByte
  pub bg1hofs: u8, // 210Dh WO - BG1HOFS - BG1Xスクロール / M7HOFS
  pub bg1vofs: u8, // 210Eh WO - BG1VOFS - BG1Yスクロール / M7VOFS

  pub vmain: u8, // 2115h WO - VMAIN   - VRAMアドレス増加レジスタ
  cgadd: u8, // 2121h WO - CGADD   - パレットアドレス
  cg_write_low: bool,
  // cgdata: Vec<u8>, // 2122h WO - CGDATA  - パレット書き込み
  cgdata: Vec<u16>, // 2122h WO - CGDATA  - パレット書き込み
  tm: u8, // 212Ch WO - TM      - メイン画面レイヤ制御
  ts: u8, // 212Dh WO - TS      - サブ画面レイヤ制御
  tmw: u8, // 212Eh WO - TMW     - Window Area Main Screen Disable
  cgwsel: u8, // 2130h WO - CGWSEL  - ColorMath制御レジスタA
  cgadsub: u8, // 2131h WO - CGADSUB - ColorMath制御レジスタB
  setini: u8, // 2133h WO - SETINI  - ディスプレイ制御レジスタ2
  // 2116h WO - VMADDL  - VRAMアドレス (下位8bit)
  // 2117h WO - VMADDH  - VRAMアドレス (上位8bit)
  vmadd: u16,
  // 2118h WO - VMDATAL - VRAMデータ書き込み (下位8bit)
  // 2119h WO - VMDATAH - VRAMデータ書き込み (上位8bit)
  vmdata: Vec<u16>,
  // 213Ch RO - OPHCT   - Hカウンタ
  ophct: u16,
  ophct_low: bool,
  // 213Dh RO - OPVCT   - Vカウンタ
  opvct: u16,
  opvct_low: bool,
  stat77: u8, // 213Eh RO - STAT77  - PPU1ステータス
  stat78: u8, // 213Fh RO - STAT78  - PPU2ステータス

  // 4200h WO - NMITIMEN- 割り込み有効化レジスタ
  nmitimen: u8,

  // 4201h WO - WRIO    - Joypad Programmable I/O Port (Open-Collector Output)
  wrio: u8,

  // 4207h WO - HTIMEL  - HIRQ座標 (下位8bits)                          (FFh)
  htimel: u8,
  // 4208h WO - HTIMEH  - HIRQ座標 (上位1bit)                           (01h)
  htimeh: u8,
  // 4209h WO - VTIMEL  - VIRQ座標 (下位8bits)                          (FFh)
  vtimel: u8,
  // 420Ah WO - VTIMEH  - VIRQ座標 (上位1bit)                           (01h)
  vtimeh: u8,

  // 4210h RO - RDNMI   - NMIフラグ (Read/Ack)
  rdnmi: u8,
  // 4211h RO - TIMEUP  - H/VタイマーIRQフラグ
  pub timeup: u8,


  // flags
  pub frame_updated: bool,
  pub screen_state: Vec<u8>,

  h_counter: u16,
  v_counter: u16,

  pub hblank_flag: bool,
  pub vblank_flag: bool,
  auto_joypad_flag: bool,

  pub hirq_flag: bool,
  pub virq_flag: bool,
  hirq_wait_flag: bool,
  virq_wait_flag: bool,
}

const WINDOW_WIDTH: usize = 256;
const WINDOW_HEIGHT: usize = 256; // 224

impl PPU {
  pub fn new() -> Self {
    Self {
      cycles: 0,

      inidisp: 0x80,
      obsel: 0x00,
      oamaddl: 0x00,
      oamaddh: 0x00,
      oamdata: 0x00,
      bgmode: 0x0F,
      mosaic: 0,
      bg1sc: 0,
      bg12nba: 0,
      bg1hofs: 0,
      bg1vofs: 0,
      vmain: 0x0F,
      cgadd: 0,
      cg_write_low: true,
      cgdata: vec![0; 512], // 256 word
      tm: 0,
      ts: 0,
      tmw: 0,
      cgwsel: 0,
      cgadsub: 0,
      setini: 0,
      vmadd: 0,
      vmdata: vec![0; 32 * 1024], // 32K Word
      ophct: 0x01FF,
      ophct_low: true,
      opvct: 0x01FF,
      opvct_low: true,
      stat77: 0x00,
      stat78: 0x00,

      nmitimen: 0x00,
      wrio: 0xFF,
      htimel: 0xFF,
      htimeh: 0x01,
      vtimel: 0xFF,
      vtimeh: 0x01,
      rdnmi: 0x02,
      timeup: 0x00,

      frame_updated: false,
      screen_state: vec![0; WINDOW_WIDTH * WINDOW_HEIGHT * 3],

      h_counter: 0,
      v_counter: 0,

      hblank_flag: false,
      vblank_flag: false,
      auto_joypad_flag: false,

      hirq_flag: false,
      virq_flag: false,
      hirq_wait_flag: false,
      virq_wait_flag: false,
    }
  }

  // 256x224px
  // 3.58MHz (21.477MHz) (1.79MHz /12、2.68MHz /8、3.58MHz /6)
  // X = 3.58MHz / 224line / 60FPS = 1ライン分の時間
  // X = 1364 / 6 = 227.33(3.58MHz換算)
  // MAX: 262スキャンライン
  // スキャンライン$E1(225: NTSC)または$F0(PAL: 240)からフレームの終わりまで実行されます。
  // 各スキャンラインの開始から約536サイクル後から40サイクルの間一時停止します。
  // 1スキャンラインあたり常に340ドット（ピクセル）??
  // 22 ～ 277 が画面に表示される。
  // Vカウンタは、NTSC モードでは 0 ～ 261
  //   1 ～ 224 の範囲が画面に表示される。
  pub fn tick(&mut self, cycles: u32) {
    self.cycles += cycles;

    let line_par_cycles = 1364;
    self.h_counter = (self.cycles / 4) as u16;

    if self.cycles > line_par_cycles {
      self.cycles -= line_par_cycles;
      self.v_counter += 1;
      self.hirq_wait_flag = true;
    }

    self.hvirq();

    // FIXME 最終的には、draw_pixel()を作って、1ピクセルづつ書くようにする。
    if self.h_counter > 277 && self.v_counter <= 224 {
      self.draw_line(self.v_counter);
    }

    // H-Blank フラグ H-Blank中はセットされている。H-Blankの外ではクリアされる。
    // セットされるタイミングは、Hカウンタが 0x121 ～ 0x122 (289 ～ 290) の時で、
    // クリアされるタイミングは、Hカウンタが 0x12 ～ 0x18 (18 ～ 24) の時。
    if !self.hblank_flag && self.h_counter >= 289 {
      self.hblank_flag = true;
    }
    if self.hblank_flag && self.h_counter >= 18 {
      self.hblank_flag = false;
    }

    if self.v_counter > 224 {
      self.set_nmi();
      self.frame_updated = true;
    }

    if self.v_counter > 261 {
      self.v_counter = 0;
      self.virq_wait_flag = true;
      self.clear_nmi();
    }

    // V-Blank フラグ V-Blank中はセットされている。
    // V-Blankの外ではクリアされる。
    // セットされるタイミングは、Vカウンタが 0xE1(225) かつ Hカウンタが 0x16 ～ 0x17 (22 ～ 23) の時で、
    // クリアされるタイミングは、Vカウンタが 0 かつ Hカウンタが 0x1E (30) の時。
    if !self.vblank_flag && self.v_counter >= 255 && self.h_counter >= 22 {
      self.vblank_flag = true;
      self.auto_joypad_flag = true;
    }
    if self.vblank_flag && self.v_counter >= 0 && self.h_counter >= 30 {
      self.vblank_flag = false;
    }

    // 自動ジョイパッドステータス 自動ジョイパッド読み込み時にセットされる。 完了時にクリアされる。
    // 典型的に、これは V-Blank 開始時にセットされ、 3 スキャンライン後に完了する。
    if self.auto_joypad_flag && self.v_counter >= 255 + 3 {
      self.auto_joypad_flag = false;
    }
  }

  fn hvirq(&mut self) {
    let hvirq = (self.nmitimen & 0x30) >> 4;
    if hvirq == 1 {
      // HIRQ
      let htime = ((self.htimeh as u16) << 8) | self.htimel as u16;
      if htime <= self.h_counter && self.hirq_wait_flag {
        self.hirq_flag = true;
        self.hirq_wait_flag = false;
      }
    } else if hvirq == 2 {
      // VIRQ
      let vtime = ((self.vtimeh as u16) << 8) | self.vtimel as u16;
      if vtime <= self.v_counter && self.virq_wait_flag {
        self.virq_flag = true;
        self.virq_wait_flag = false;
      }
    } else if hvirq == 3 {
      // HVIRQ
      let htime = ((self.htimeh as u16) << 8) | self.htimel as u16;
      let vtime = ((self.vtimeh as u16) << 8) | self.vtimel as u16;

      if htime <= self.h_counter && vtime <= self.v_counter && self.hirq_wait_flag && self.virq_wait_flag {
        self.hirq_flag = true;
        self.virq_flag = true;

        self.hirq_wait_flag = false;
        self.virq_wait_flag = false;
      }
    }
  }

  fn set_nmi(&mut self) {
    self.rdnmi = self.rdnmi | 0x80;
  }

  fn clear_nmi(&mut self) {
    self.rdnmi = self.rdnmi & 0x0F;
    self.vblank_flag = false;
  }

  fn bg1_tilemaps(&mut self, y: u32) -> &[u16] {
    let base = (((self.bg1sc & 0xFC) as u32) << 8) as usize;
    let offset: usize = y as usize / 8;
    let base = base + 32 * offset;
    &self.vmdata[base..=base + 32]
  }

  fn bg1tile(&mut self, tileindex: u16) -> &[u16] {
    // bgモードみる
    //  -> いまは2bpp固定
    let tilesize: usize = 2 /*bpp*/ * 8 /* 8x8mode */ / 2 /*byte to word */;
    let base = (((self.bg12nba & 0x0F) as u32) << 12) as usize;
    let addr = base + tilesize * tileindex as usize;
    let data = &self.vmdata[addr..=(addr + tilesize)];
    data
  }

  fn palette(&mut self, palette_size: u16) -> Vec<[u8; 3]> {
    // 2bpp固定
    let base =  palette_size as usize;
    let palette_size = 4; // 2bpp
    let data = &self.cgdata[base..=base + palette_size];
    let mut res: Vec<[u8; 3]> = vec![];
    for v in data {
      res.push(bgr555_to_rgb888(*v));
    }
    res
  }

  fn draw_line(&mut self, scanline: u16) {
    // TODO
    // BG1HOFS = x offset
    // BG1VOFS = y offset
    let offset_x: u32 = 0;
    let offset_y: u32 = 0;

    let ox = offset_x;
    let oy = offset_y + scanline as u32;

    let tilemaps = self.bg1_tilemaps(oy).to_vec();
    let tile_y: usize = oy as usize % 8;

    let mut draw_x: usize = 0;
    for (i, tilemap) in tilemaps.iter().enumerate() {
      if (i as usize * 8) < ox as usize {
        continue;
      }
      if i as usize * 8 + ox as usize > WINDOW_WIDTH {
        continue;
      }

      let tileindex = tilemap & 0x02FF;
      let palette_select = (tilemap & 0x1C00) >> 10;

      let tile = self.bg1tile(tileindex).to_vec();
      let palette = self.palette(palette_select);

      let line = tile[tile_y];
      for x in 0..8 {

        if draw_x > WINDOW_WIDTH {
          break;
        }

        let mask = 0x80 >> x;
        let palette_index = ((line & (mask << 8)) >> (15 - x)) + ((line & mask) >> (7 - x));
        let rgb = palette[palette_index as usize];

        let base_index = (scanline as usize * WINDOW_WIDTH + draw_x) * 3;
        self.screen_state[base_index + 0] = rgb[0];
        self.screen_state[base_index + 1] = rgb[1];
        self.screen_state[base_index + 2] = rgb[2];
        draw_x += 1;
      }
    }
  }

  fn increment_timing(&self) -> u8 {
    (self.vmain & 0x80) >> 7
  }

  fn increment_vmadd(&mut self) {
    // 7 上位/下位バイトにアクセスした後、VRAM アドレスをインクリメントします (0=下位、1=上位)
    // 6-4 未使用
    // TODO アドレス変換はまだ未実装。
    // 3-2 アドレス変換 (0..3 = 0 ビット/なし、8 ビット、9 ビット、10 ビット)
    // 1-0 アドレスインクリメント ステップ (0..3 = ワード アドレスを 1、32、128、128 ずつインクリメント)
    // let timing = (self.vmain & 0x80) >> 7;
    // let address_transfer = (self.vmain & 0x0C) >> 2;
    let step = self.vmain & 0x03;
    self.vmadd += match step {
      0 => 1,
      1 => 32,
      2 | 3 => 128,
      _ => panic!("invalid address increment step!"),
    }
  }

  fn write_vmdatal(&mut self, data: u8) {
    let vmadd = (self.vmadd & 0x7FFF) as usize;
    self.vmdata[vmadd] = self.replace_lsb(self.vmdata[vmadd], data);
    println!("write_vmdatal({:02X}) addr: {:04X}, data: {:04X}", data, vmadd, self.vmdata[vmadd]);
    if self.increment_timing() == 0 {
      self.increment_vmadd();
    }
  }

  fn write_vmdatah(&mut self, data: u8) {
    let vmadd = (self.vmadd & 0x7FFF) as usize;
    self.vmdata[vmadd] = self.replace_msb(self.vmdata[vmadd], data);
    println!("write_vmdatah({:02X}) addr: {:04X}, data: {:04X}", data, vmadd, self.vmdata[vmadd]);
    if self.increment_timing() == 1 {
      self.increment_vmadd();
    }
  }

  fn replace_lsb(&self, data: u16, value: u8) -> u16 {
    (data & 0xFF00) | (value as u16)
  }

  fn replace_msb(&self, data: u16, value: u8) -> u16 {
    (data & 0x00FF) | ((value as u16) << 8)
  }

  pub fn write(&mut self, addr: u16, data: u8) {
    match addr {
      0x2100 => self.inidisp = data,
      0x2101 => self.obsel = data,
      0x2102 => self.oamaddl = data,
      0x2103 => self.oamaddh = data,
      0x2104 => self.oamdata = data,
      0x2105 => {
        println!("BGMODE: {:02X}", data);
        self.bgmode = data
      },
      0x2106 => self.mosaic = data,
      0x2107 => self.bg1sc = data,
      0x2108..=0x210A => {} // FIXME BG画面設定
      0x210B => self.bg12nba = data, // 04 => BG1 4 x 0x2000 ?
      0x210C => {}, // FIXME BG3,4タイルデータアドレス
      0x210D => {
        println!("BG1HOFS: {:02X}", data);
        self.bg1hofs = data
      }
      0x210E => self.bg1vofs = data,
      0x210F..=0x2114 => {}, // FIXME BG2,3,4Xスクロール
      0x2115 => self.vmain = data,
      0x211A..=0x2120 => {}, // FIXME Mode 7関係
      0x2121 => {
        self.cgadd = data;
        self.cg_write_low = true;
      },
      0x2122 => {
        println!("write cgdata(low={}) {:02X} => {:02X}", self.cg_write_low, self.cgadd, data);
        if self.cg_write_low {
          self.cgdata[self.cgadd as usize] = self.replace_lsb(self.cgdata[self.cgadd as usize], data);
        } else {
          self.cgdata[self.cgadd as usize] = self.replace_msb(self.cgdata[self.cgadd as usize], data);
          self.cgadd = self.cgadd.wrapping_add(1);
        }
        self.cg_write_low = !self.cg_write_low;
      },
      0x2123..=0x0212B => {},
      0x212C => self.tm = data,
      0x212D => self.ts = data,
      0x212E => self.tmw = data,
      0x212F => {} // FIXME Window Area Sub Screen Disable
      0x2130 => self.cgwsel = data,
      0x2131 => self.cgadsub = data,
      0x2132 => {} // FIXME Color Math Sub Screen Backdrop Color
      0x2133 => self.setini = data,
      0x2116 => {
        self.vmadd = self.replace_lsb(self.vmadd, data);
      }
      0x2117 => {
        self.vmadd = self.replace_msb(self.vmadd, data);
      }
      0x2118 => self.write_vmdatal(data),
      0x2119 => self.write_vmdatah(data),
      0x4200 => {
        self.nmitimen = data;
        if (self.nmitimen & 0x30) == 0 {
          self.timeup = self.timeup & 0x7F;
        }
      }
      0x4201 => self.wrio = data,
      _ => panic!("not implement PPU::write({:04X}, {:02X})", addr, data),
    }
  }

  pub fn read(&mut self, addr: u16) -> u8 {
    match addr {
      0x2137 => { // 2137h RO - SLHV    - H/Vカウンタラッチ
        if self.wrio & 0x80 != 0 {
          self.ophct = self.h_counter;
          self.opvct = self.v_counter;
        }
        0 // オープンバス
      }
      0x213C => {
        let val = if self.ophct_low {
          self.ophct & 0x00FF
        } else {
          (self.ophct & 0xFF00) >> 8
        };
        self.ophct_low = !self.ophct_low;
        val as u8
      }
      0x213D => {
        let val = if self.opvct_low {
          self.opvct & 0x00FF
        } else {
          (self.opvct & 0xFF00) >> 8
        };
        self.opvct_low = !self.opvct_low;
        val as u8
      }
      0x213E => {
        self.stat77
      }
      0x213F => {
        self.stat78
      }
      0x4210 => {
        let res = self.rdnmi;
        self.clear_nmi();
        res
      }
      0x4211 => {
        let v = self.timeup;
        self.timeup = self.timeup & 0x7F;
        v
      }
      0x4212 => {
        (if self.vblank_flag { 0x80 } else { 0x00 })
        | (if self.hblank_flag { 0x40 } else { 0x00 })
        | (if self.auto_joypad_flag { 0x01 } else { 0x00 })
      }
      _ => panic!("not implement PPU::read({:04X})", addr),
    }
  }

}
