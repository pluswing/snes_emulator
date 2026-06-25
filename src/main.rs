mod cpu;
mod opscodes;
mod ppu;
mod bus;
mod cartridge;

use cartridge::Cartridge;
use bus::Bus;
use cpu::CPU;
use ppu::PPU;

use sdl3::pixels::{Color, PixelFormat};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use std::time::Duration;

fn main() {

  let sdl_context = sdl3::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem.window("SNES Emulator", 512, 448)
        .position_centered()
        .build()
        .unwrap();

  let mut canvas = window.into_canvas();

  let creator = canvas.texture_creator();
  let mut texture = creator
    .create_texture_target(PixelFormat::RGB24, 512, 448)
    .unwrap();

  canvas.set_draw_color(Color::RGB(0, 0, 0));
  canvas.clear();
  canvas.present();

  let mut event_pump = sdl_context.event_pump().unwrap();

  let cartridge = Cartridge::new("rom/SNES/TEST/cputest.sfc");
  // let mut cartridge = Cartridge::new("rom/SNES/ROM/CHRONO TRIGGER/50/Chrono Trigger (Japan).sfc");
  let mut ppu = PPU::new();
  let mut bus = Bus::new(
    ppu,
    cartridge,
  );
  let mut cpu = CPU::new(bus);

  cpu.reset();

  'running: loop {
    cpu.run();
    if cpu.bus.ppu.frame_updated {
      cpu.bus.ppu.frame_updated = false;

      for event in event_pump.poll_iter() {
        match event {
          Event::Quit {..} |
          Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
              break 'running
          },
          _ => {}
        }
      }

      canvas.clear();
      texture.update(None, &cpu.bus.ppu.screen_state, 256 * 3).unwrap();
      canvas.copy(&texture, None, None).unwrap();
      canvas.present();
    }

    // ::std::thread::sleep(Duration::new(0,   1_000_000_000u32 / 60));
  }
}
