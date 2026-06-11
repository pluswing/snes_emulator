mod cpu;
mod opscodes;
mod ppu;
mod bus;
mod cartridge;

use cartridge::Cartridge;
use bus::Bus;
use cpu::CPU;
use ppu::PPU;

fn main() {
  // let cartridge = Cartridge::new("rom/SNES/TEST/cputest.sfc");
  let mut cartridge = Cartridge::new("rom/SNES/ROM/CHRONO TRIGGER/50/Chrono Trigger (Japan).sfc");
  let mut ppu = PPU::new();
  let mut bus = Bus::new(
    ppu,
    cartridge,
  );
  let mut cpu = CPU::new(bus);

  loop {
    cpu.run();
  }
}
