use serde::{Serialize, Deserialize};
use std::fs;
use std::fmt;

use crate::apu::{AddressingMode, APU};

mod apu;

/*
{
  "name": "00 0000",
  "initial": {
    "pc": 30256,
    "a": 56,
    "x": 78,
    "y": 127,
    "sp": 236,
    "psw": 145,
    "ram": [
      [
        30256,
        0
      ]
    ]
  },
  "final": {
    // initialと同じ
  },
  "cycles": [
    [
      30256,
      0,
      "read"
    ],
    [
      30257,
      null,
      "read"
    ]
  ]
}
 */

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct TestCaseRegisterData {
  #[serde(rename = "pc")]
  Pc: u16,
  #[serde(rename = "a")]
  A: u8,
  #[serde(rename = "x")]
  X: u8,
  #[serde(rename = "y")]
  Y: u8,
  #[serde(rename = "sp")]
  Sp: u8,
  #[serde(rename = "psw")]
  Psw: u8,
  #[serde(rename = "ram")]
  Ram: Vec<(u16, u8)>,
}

impl fmt::Debug for TestCaseRegisterData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A:{:02X} X:{:02X} Y:{:02X} ", self.A, self.X, self.Y);
        write!(f, "PC:{:04X} SP:{:02X} PSW:{:0>8b} ", self.Pc, self.Sp, self.Psw);
        write!(f, "RAM:");
        for r in &self.Ram {
          write!(f, " {:06X} {:02X},", r.0, r.1);
        }
        Result::Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct TestCaseData {
    name: String,
    #[serde(rename = "initial")]
    Initial: TestCaseRegisterData,
    #[serde(rename = "final")]
    Final: TestCaseRegisterData,
    #[serde(rename = "cycles")]
    Cycles: Vec<(Option<u16>, Option<u8>, String)>,
}

fn main() {
    let targets = [
      "cd"
    ];
    // let targets = testcases_by_name("MVN");
    // let targets = testcases_by_addressing_mode(&AddressingMode::Absolute_Indexed_by_X);
    // let targets = testcases();

    for target in targets {
      let input_fn = fs::read_to_string(format!("tests/apu_cases/{}.json", target)).expect("JSON Read Failed.");
      let deserialized: Vec<TestCaseData> = serde_json::from_str(&input_fn).unwrap();

      let mut apu = APU::new();

      for data in &deserialized {
        // apuにInitialをセット
        apu.program_counter = data.Initial.Pc;
        apu.stack_pointer = data.Initial.Sp;
        apu.program_status = data.Initial.Psw;
        apu.set_register_a(data.Initial.A);
        apu.set_register_x(data.Initial.X);
        apu.set_register_y(data.Initial.Y);
        for d in &data.Initial.Ram {
          apu.mem_write(d.0, d.1);
        }

        // cpuを1命令分動かす（？）
        let opcode = apu.mem_read(apu.program_counter);
        let arg1 = apu.mem_read(apu.program_counter+1);
        let arg2 = apu.mem_read(apu.program_counter+2);
        println!("---------------------");
        println!("RUN name: \"{}\" {:02X} {:02X} {:02X}", data.name, opcode, arg1, arg2);
        // println!("cycles:");
        // for c in &data.Cycles {
        //   if c.1.is_some() {
        //     print!("{:06X} {:02X} {}, ", c.0, c.1.unwrap(), c.2)
        //   } else {
        //     print!("{:06X} __ {}, ", c.0, c.2)
        //   }
        // }
        println!("initial:                         NVMXDIZC\n{:?}", data.Initial);
        println!("expected:                        NVMXDIZC\n{:?}", data.Final);
        apu.run();
        // println!("A initial: {:04X}, expected: {:04X}, actual: {:04X}", data.Initial.A, data.Final.A, cpu.register_a);

        // cpuの状態とFinalが合っているか確認
        assert_eq!(apu.program_counter, data.Final.Pc, "[PC] {:04X} {:04X}", apu.program_counter, data.Final.Pc);
        assert_eq!(apu.stack_pointer, data.Final.Sp, "[SP] {:04X} {:04X}", apu.stack_pointer, data.Final.Sp);
        assert_eq!(apu.get_register_a(), data.Final.A, "[A] {:04X} {:04X}", apu.get_register_a(), data.Final.A);
        assert_eq!(apu.get_register_x(), data.Final.X, "[X] {:04X} {:04X}", apu.get_register_x(), data.Final.X);
        assert_eq!(apu.get_register_y(), data.Final.Y, "[Y] {:04X} {:04X}", apu.get_register_y(), data.Final.Y);
        assert_eq!(apu.program_status, data.Final.Psw, "[Psw] {:0>8b} {:0>8b}", apu.program_status, data.Final.Psw);
        for d in &data.Final.Ram {
          assert_eq!(apu.mem_read(d.0), d.1, "[MEM] {:06X} {:02X} {:02X}", d.0, apu.mem_read(d.0), d.1);
        }
        // TODO data.Cycles
      }
    }

    println!("OK!");
}
