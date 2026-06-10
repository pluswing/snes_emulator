mod cpu;
mod opscodes;
mod ppu;
mod bus;
mod cartridge;

use cartridge::Cartridge;

fn main() {
  let cartridge = Cartridge::new("rom/SNES/TEST/cputest.sfc");
  // let cpu = CPU::new();
}
