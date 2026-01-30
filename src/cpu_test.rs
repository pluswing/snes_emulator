use serde::{Serialize, Deserialize};
use std::fs;
use std::io::prelude::*;
use std::fmt;

use crate::cpu::CPU;

mod cpu;
mod opscodes;

/*
{
  "name": "a9 n 1",
  "initial": {
    "pc": 64793,
    "s": 30012,
    "p": 182,
    "a": 7881,
    "x": 105,
    "y": 80,
    "dbr": 16,
    "d": 2983,
    "pbr": 98,
    "e": 0,
    "ram": [
      [6487322, 251],
      [6487321, 169]
    ]
  },
  "final": // initialと同じ,
  "cycles": [
    [6487321, 169, "dp-r-mx-"],
    [6487322, 251, "-p-r-mx-"]
  ]
}
 */

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct TestCaseRegisterData {
  #[serde(rename = "pc")]
  Pc: u16,
  #[serde(rename = "s")]
  S: u16,
  #[serde(rename = "p")]
  P: u8,
  #[serde(rename = "a")]
  A: u16,
  #[serde(rename = "x")]
  X: u16,
  #[serde(rename = "y")]
  Y: u16,
  #[serde(rename = "dbr")]
  Dbr: u8,
  #[serde(rename = "d")]
  D: u16,// ??
  #[serde(rename = "pbr")]
  Pbr: u8,
  #[serde(rename = "e")]
  E: u8,
  #[serde(rename = "ram")]
  Ram: Vec<(u32, u8)>,
}

impl fmt::Debug for TestCaseRegisterData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A:   {:04X} X:  {:04X} Y: {:04X}  ", self.A, self.X, self.Y);
        write!(f, "PBR: {:02X} PC: {:04X} => {:06X}\n", self.Pbr, self.Pc, (self.Pbr as u32) << 16 | self.Pc as u32);
        write!(f, "S:   {:04X} P:  {:0>8b} E: {} ", self.S, self.P, self.E);
        write!(f, "DBR: {:02X} DP: {:04X}\n", self.Dbr, self.D);
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
    Cycles: Vec<(u32, Option<u8>, String)>,
}

fn main() {
    let targets = [
      "a9.e", // LDA Immediate
      "a9.n",
      "ad.e", // LDA Absolute
      "ad.n",
      "af.e", // LDA Absolute Long
      "af.n",
      "a5.e", // LDA Direct Page (=6502: Zero Page)
      "a5.n",
      "b2.e", // LDA Direct Page Indirect
    ];

    for target in targets {
      let input_fn = fs::read_to_string(format!("tests/cases/{}.json", target)).expect("JSON Read Failed.");
      let deserialized: Vec<TestCaseData> = serde_json::from_str(&input_fn).unwrap();

      let mut cpu = CPU::new();

      for data in &deserialized {
        // cpuにInitialをセット
        cpu.program_counter = data.Initial.Pc;
        cpu.stack_pointer = data.Initial.S;
        cpu.status = data.Initial.P;
        cpu.register_a = data.Initial.A;
        cpu.register_x = data.Initial.X;
        cpu.register_y = data.Initial.Y;
        cpu.data_bank = data.Initial.Dbr;
        cpu.direct_page = data.Initial.D;
        cpu.program_bank = data.Initial.Pbr;
        cpu.mode = data.Initial.E;
        for d in &data.Initial.Ram {
          cpu.mem_write(d.0, d.1);
        }

        // cpuを1命令分動かす（？）
        let opcode = cpu.mem_read(cpu.pc());
        let arg1 = cpu.mem_read(cpu.pc()+1);
        let arg2 = cpu.mem_read(cpu.pc()+2);
        println!("---------------------");
        println!("RUN name: \"{}\" {:02X} {:02X} {:02X}", data.name, opcode, arg1, arg2);
        cpu.run();
        // println!("A initial: {:04X}, expected: {:04X}, actual: {:04X}", data.Initial.A, data.Final.A, cpu.register_a);

        println!("initial:      NVMXDIZC\n{:?}", data.Initial);
        println!("expected:     NVMXDIZC\n{:?}", data.Final);

        // cpuの状態とFinalが合っているか確認
        assert_eq!(cpu.program_counter, data.Final.Pc, "[PC] {:04X} {:04X}", cpu.program_counter, data.Final.Pc);
        assert_eq!(cpu.stack_pointer, data.Final.S, "[S] {:04X} {:04X}", cpu.stack_pointer, data.Final.S);
        assert_eq!(cpu.register_a, data.Final.A, "[A] {:04X} {:04X}", cpu.register_a, data.Final.A);
        assert_eq!(cpu.status, data.Final.P, "[P] {:0>8b} {:0>8b}", cpu.status, data.Final.P);
        assert_eq!(cpu.register_x, data.Final.X, "[X] {:04X} {:04X}", cpu.register_x, data.Final.X);
        assert_eq!(cpu.register_y, data.Final.Y, "[Y] {:04X} {:04X}", cpu.register_y, data.Final.Y);
        assert_eq!(cpu.data_bank, data.Final.Dbr, "[DBR] {:02X} {:02X}", cpu.data_bank, data.Final.Dbr);
        assert_eq!(cpu.direct_page, data.Final.D, "[DP] {:04X} {:04X}", cpu.direct_page, data.Final.D);
        assert_eq!(cpu.program_bank, data.Final.Pbr, "[PBR] {:02X} {:02X}", cpu.program_bank, data.Final.Pbr);
        assert_eq!(cpu.mode, data.Final.E);
        for d in &data.Final.Ram {
          assert_eq!(cpu.mem_read(d.0), d.1, "[MEM] {:06X} {:02X} {:02X}", d.0, cpu.mem_read(d.0), d.1);
        }
        // TODO data.Cycles
      }
    }

    println!("OK!");
}
