mod cpu;
mod opscodes;
mod ppu;
mod bus;
mod cartridge;
mod apu;

use cartridge::Cartridge;
use bus::Bus;
use cpu::CPU;
use ppu::PPU;

use sdl3::pixels::{Color, PixelFormat};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;

use crate::bus::Mem;

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

  // debug
  let window2 = video_subsystem.window("debug", 8*32, 8*32)
        .position_centered()
        .build()
        .unwrap();
  let mut canvas2 = window2.into_canvas();
  let creator2 = canvas2.texture_creator();
  let mut texture2 = creator2
    .create_texture_target(PixelFormat::RGB24, 8*32, 8*32)
    .unwrap();
  // debug

  let mut event_pump = sdl_context.event_pump().unwrap();

  let cartridge = Cartridge::new("rom/SNES/TEST/cputest-full.sfc");
  // let cartridge = Cartridge::new("rom/SNES/TEST/gradient-test.sfc");
  // let cartridge = Cartridge::new("rom/SNES/TEST/twoship.sfc");
  // let cartridge = Cartridge::new("rom/SNES/TEST/ppubusact.sfc");
  // let mut cartridge = Cartridge::new("rom/SNES/ROM/CHRONO TRIGGER/50/Chrono Trigger (Japan).sfc");
  // let cartridge = Cartridge::new("rom/SNES/ROM/SUPERMARIO COLLECTION/61/Super Mario Collection (Japan).sfc");
  // let cartridge = Cartridge::new("rom/SNES/ROM/SUPER BOMBERMAN/77/Super Bomberman (Japan).sfc");
  // let cartridge = Cartridge::new("rom/SNES/ROM/DRAGONQUEST5/56/Dragon Quest V - Tenkuu no Hanayome (Japan).sfc");
  let ppu = PPU::new();
  let bus = Bus::new(
    ppu,
    cartridge,
  );
  let mut cpu = CPU::new(bus);

  cpu.reset();
  let mut frame = 0;

  'running: loop {
    cpu.run();
    if cpu.bus.ppu.frame_updated {
      cpu.bus.ppu.frame_updated = false;
      frame += 1;

      for event in event_pump.poll_iter() {
        match event {
          Event::Window {
              win_event: WindowEvent::CloseRequested,
              ..
          } |
          Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
              break 'running
          },
          Event::KeyDown { keycode: Some(Keycode::A), .. } => {
            cpu.bus.joypad1 = 0x0080;
          }
          Event::KeyUp { keycode: Some(Keycode::A), .. } => {
            cpu.bus.joypad1 = 0x0000;
          }
          _ => {}
        }
      }

      canvas.clear();
      texture.update(None, &cpu.bus.ppu.screen_state, 256 * 3).unwrap();
      canvas.copy(&texture, None, None).unwrap();

      canvas.set_draw_color((255, 0, 0));
      canvas.draw_debug_text(format!("FRAME: {}", frame).as_str(), (0, 100)).unwrap();

      canvas.present();

      // debug
      canvas2.clear();
      texture2.update(None, &cpu.bus.ppu.debug_bg1_tiles, 8*32 * 3).unwrap();
      canvas2.copy(&texture2, None, None).unwrap();
      canvas2.present();
      // debug

    }

    // ::std::thread::sleep(Duration::new(0,   1_000_000_000u32 / 60));
  }
}
