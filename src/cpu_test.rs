use serde::{Serialize, Deserialize};
use std::fs;
use std::io::prelude::*;

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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct TestCaseData {
    name: String,
    #[serde(rename = "initial")]
    Initial: TestCaseRegisterData,
    #[serde(rename = "final")]
    Final: TestCaseRegisterData,
    #[serde(rename = "cycles")]
    Cycles: Vec<(u32, u8, String)>,
}

fn main() {
    println!("CPU TEST!");
    // TODO a9.n を実行時引数でもらう。
    let target = "tests/cases/a9.n.json";

    let input_fn = fs::read_to_string(target).expect("JSON Read Failed.");
    let deserialized: Vec<TestCaseData> = serde_json::from_str(&input_fn).unwrap();

    let mut cpu = CPU::new();

    for data in &deserialized {
      println!("RUN {}", data.name);
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
      println!("OP: {:02X}", opcode);
      cpu.run();

      // cpuの状態とFinalが合っているか確認
      assert_eq!(cpu.program_counter, data.Final.Pc);
      assert_eq!(cpu.stack_pointer, data.Final.S);
      assert_eq!(cpu.status, data.Final.P);
      assert_eq!(cpu.register_a, data.Final.A);
      assert_eq!(cpu.register_x, data.Final.X);
      assert_eq!(cpu.register_y, data.Final.Y);
      assert_eq!(cpu.data_bank, data.Final.Dbr);
      assert_eq!(cpu.direct_page, data.Final.D);
      assert_eq!(cpu.program_bank, data.Final.Pbr);
      assert_eq!(cpu.mode, data.Final.E);
      for d in &data.Final.Ram {
        assert_eq!(cpu.mem_read(d.0), d.1);
      }
      // TODO data.Cycles
    }
}
