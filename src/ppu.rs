use core::panic;

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
  scanline: u8,
  // registers
  pub inidisp: u8, // 2100h WO - INIDISP - ディスプレイ制御レジスタ1
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
  // 213Dh RO - OPVCT   - Vカウンタ

  // 4210h RO - RDNMI   - NMIフラグ (Read/Ack)
  rdnmi: u8,

  // flags
  pub frame_updated: bool,
  pub screen_state: Vec<u8>,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      cycles: 0,
      scanline: 0,
      inidisp: 0x80,
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
      rdnmi: 0x02,

      frame_updated: false,
      screen_state: vec![0; 256 * 256 * 3], // 224
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
  pub fn tick(&mut self, cycles: u8) {
    self.cycles += cycles as u32;

    let line_par_cycles = 227;
    if self.cycles > line_par_cycles {
      self.cycles -= line_par_cycles;
      self.scanline += 1;

      if self.scanline <= 224 {
        // HBlank割り込み = 1行描画
        self.interrupt_hbank();
      }
    }
    if self.scanline > 224 {
      // VBlank割り込み = WAIT
      self.draw_line(self.scanline);
      self.interrupt_vbank();
      self.frame_updated = true;
    }
    if self.scanline > 234 {
      // FIXME
      self.scanline = 0;
      self.clear_nmi();
    }
  }

  fn set_nmi(&mut self) {
    self.rdnmi = self.rdnmi | 0x80;
  }

  fn clear_nmi(&mut self) {
    self.rdnmi = self.rdnmi & 0x0F;
  }

  fn interrupt_hbank(&mut self) {

  }

  fn interrupt_vbank(&mut self) {
    self.set_nmi()
  }

  fn bg1tilemaps(&mut self) -> &[u16] {
    let base = (((self.bg1sc & 0xFC) as u32) << 8) as usize;
    let tilemaps = &self.vmdata[base..=(base + 32 * 32)];
    tilemaps
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

  fn draw_line(&mut self, scanline: u8) {
    let tilemaps = self.bg1tilemaps().to_vec();
    // [33] = 0052, 0075, 006E, 006E, 0069, 006E, 0067, 0020, 0074, 0065, 0073
    for (i, tilemap) in tilemaps.iter().enumerate() {
      let tx = (i % 32) * 8;
      let ty = (i / 32) * 8;
      // tilemap = VHPC CCTT TTTT TTTT
      let tileindex = tilemap & 0x02FF;
      let palette_select = (tilemap & 0x1C00) >> 10;

      // tileをとって
      let tile = self.bg1tile(tileindex).to_vec();
      // paletteをとって
      // ピクセルの色が決まって
      let palette = self.palette(palette_select);

      // かく！
      for (y, line) in tile.iter().enumerate() {
        for x in 0..8 {
          let mask = 0x80 >> x;
          let palette_index = ((line & (mask << 8)) >> (15 - x)) + ((line & mask) >> (7 - x));
          let rgb = palette[palette_index as usize];
          if ty > 230 {
            continue;
          }
          let base_index = ((ty + y) * 256 + x + tx) * 3;
          self.screen_state[base_index + 0] = rgb[0];
          self.screen_state[base_index + 1] = rgb[1];
          self.screen_state[base_index + 2] = rgb[2];
        }
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
    let step = (self.vmain & 0x03);
    self.vmadd += match step {
      0 => 1,
      1 => 32,
      2 | 3 => 128,
      _ => panic!("invalid address increment step!"),
    }
  }

  fn write_vmdatal(&mut self, data: u8) {
    let vmadd = (self.vmadd & 0x7F) as usize;
    self.vmdata[vmadd] = self.replace_lsb(self.vmdata[vmadd], data);
    println!("write_vmdatal({:02X}) addr: {:04X}, data: {:04X}", data, vmadd, self.vmdata[vmadd]);
    if self.increment_timing() == 0 {
      self.increment_vmadd();
    }
  }

  fn write_vmdatah(&mut self, data: u8) {
    let vmadd = (self.vmadd & 0x7F) as usize;
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
      0x2105 => {
        println!("BGMODE: {:02X}", data);
        self.bgmode = data
      },
      0x2106 => self.mosaic = data,
      0x2107 => self.bg1sc = data,
      0x210B => self.bg12nba = data, // 04 => BG1 4 x 0x2000 ?
      0x210D => {
        println!("BG1HOFS: {:02X}", data);
        self.bg1hofs = data
      }
      0x210E => self.bg1vofs = data,
      0x2115 => self.vmain = data,
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
          self.cgadd += 1;
        }
        self.cg_write_low = !self.cg_write_low;
      },
      0x212C => self.tm = data,
      0x212D => self.ts = data,
      0x212E => self.tmw = data,
      0x2130 => self.cgwsel = data,
      0x2131 => self.cgadsub = data,
      0x2133 => self.setini = data,
      0x2116 => {
        self.vmadd = self.replace_lsb(self.vmadd, data);
      }
      0x2117 => {
        self.vmadd = self.replace_msb(self.vmadd, data);
      }
      0x2118 => self.write_vmdatal(data),
      0x2119 => self.write_vmdatah(data),
      _ => panic!("not implement PPU::write({:04X}, {:02X})", addr, data),
    }
  }


  pub fn read(&mut self, addr: u16) -> u8 {
    match addr {
      0x2137 => { // 2137h RO - SLHV    - H/Vカウンタラッチ
        // 今書いているx, yの座標をとってきて、
        // 213Ch RO - OPHCT   - Hカウンタ                       (01FFh)
        // 213Dh RO - OPVCT   - Vカウンタ                       (01FFh)
        // にセットする
        0 // TODO オープンバス
      }
      0x4210 => {
        let res = self.rdnmi;
        self.clear_nmi();
        res
      }
      _ => panic!("not implement PPU::read({:04X})", addr),
    }
  }

}
