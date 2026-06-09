pub struct PPU {
  cycles: u32,
  scanline: u8,
  // registers
  pub ctrl: u8, // $2000
  pub mask: u8, // $2001
  pub status: u8, // $2002
}

impl PPU {
  pub fn new() -> Self {
    Self {
      cycles: 0,
      scanline: 0,
      ctrl: 0,
      mask: 0,
      status: 0,
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

    if self.cycles > X {
      self.cycles -= X;
      self.scanline += 1;
      // HBlank割り込み = 1行描画
    }
    if self.scanline > 224 {
      // VBlank割り込み = WAIT
    }
    if self.scanline > 234 {
      // FIXME
      self.scanline = 0;
    }
  }

  pub fn write_ctrl(&mut self, value: u8) {
    self.ctrl = value;
  }

  pub fn read_ctrl(&self) -> u8 {
    self.ctrl
  }

}
