use serde::{Serialize, Deserialize};
use std::fs;
use std::fmt;

use crate::apu::{Args, APU};

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
      "00",
      "01",
      "02",
      "03",
      "04",
      "05",
      "06",
      "07",
      "08",
      "09",
      "0A",
      "0B",
      "0C",
      "0D",
      "0E",
      "0F",
      "10",
      "11",
      "12",
      "13",
      "14",
      "15",
      "16",
      "17",
      "18",
      "19",
      "1A",
      "1B",
      "1C",
      "1D",
      "1E",
      "1F",
      "20",
      "21",
      "22",
      "23",
      "24",
      "25",
      "26",
      "27",
      "28",
      "29",
      "2A",
      "2B",
      "2C",
      "2D",
      "2E",
      "2F",
      "30",
      "31",
      "32",
      "33",
      "34",
      "35",
      "36",
      "37",
      "38",
      "39",
      "3A",
      "3B",
      "3C",
      "3D",
      "3E",
      "3F",
      "40",
      "41",
      "42",
      "43",
      "44",
      "45",
      "46",
      "47",
      "48",
      "49",
      "4A",
      "4B",
      "4C",
      "4D",
      "4E",
      "4F",
      "50",
      "51",
      "52",
      "53",
      "54",
      "55",
      "56",
      "57",
      "58",
      "59",
      "5A",
      "5B",
      "5C",
      "5D",
      "5E",
      "5F",
      "60",
      "61",
      "62",
      "63",
      "64",
      "65",
      "66",
      "67",
      "68",
      "69",
      "6A",
      "6B",
      "6C",
      "6D",
      "6E",
      "6F",
      "70",
      "71",
      "72",
      "73",
      "74",
      "75",
      "76",
      "77",
      "78",
      "79",
      "7A",
      "7B",
      "7C",
      "7D",
      "7E",
      "7F",
      "80",
      "81",
      "82",
      "83",
      "84",
      "85",
      "86",
      "87",
      "88",
      "89",
      "8A",
      "8B",
      "8C",
      "8D",
      "8E",
      "8F",
      "90",
      "91",
      "92",
      "93",
      "94",
      "95",
      "96",
      "97",
      "98",
      "99",
      "9A",
      "9B",
      "9C",
      "9D",
      "9E",
      "9F",
      "A0",
      "A1",
      "A2",
      "A3",
      "A4",
      "A5",
      "A6",
      "A7",
      "A8",
      "A9",
      "AA",
      "AB",
      "AC",
      "AD",
      "AE",
      "AF",
      "B0",
      "B1",
      "B2",
      "B3",
      "B4",
      "B5",
      "B6",
      "B7",
      "B8",
      "B9",
      "BA",
      "BB",
      "BC",
      "BD",
      "BE",
      "BF",
      "C0",
      "C1",
      "C2",
      "C3",
      "C4",
      "C5",
      "C6",
      "C7",
      "C8",
      "C9",
      "CA",
      "CB",
      "CC",
      "CD",
      "CE",
      "CF",
      "D0",
      "D1",
      "D2",
      "D3",
      "D4",
      "D5",
      "D6",
      "D7",
      "D8",
      "D9",
      "DA",
      "DB",
      "DC",
      "DD",
      "DE",
      "DF",
      "E0",
      "E1",
      "E2",
      "E3",
      "E4",
      "E5",
      "E6",
      "E7",
      "E8",
      "E9",
      "EA",
      "EB",
      "EC",
      "ED",
      "EE",
      "EF",
      "F0",
      "F1",
      "F2",
      "F3",
      "F4",
      "F5",
      "F6",
      "F7",
      "F8",
      "F9",
      "FA",
      "FB",
      "FC",
      "FD",
      "FE",
      "FF",
    ];
    // let targets = testcases_by_name("MVN");
    // let targets = testcases_by_addressing_mode(&Args::Absolute_Indexed_by_X);
    // let targets = testcases();

    for target in targets {
      let input_fn = fs::read_to_string(format!("tests/apu_cases/{}.json", target)).expect("JSON Read Failed.");
      let deserialized: Vec<TestCaseData> = serde_json::from_str(&input_fn).unwrap();

      let mut apu = APU::unit_test();

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
        let arg1 = apu.mem_read(apu.program_counter.wrapping_add(1));
        let arg2 = apu.mem_read(apu.program_counter.wrapping_add(2));
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
        println!("initial:                         NVD-H-ZC\n{:?}", data.Initial);
        println!("expected:                        NVD-H-ZC\n{:?}", data.Final);
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
