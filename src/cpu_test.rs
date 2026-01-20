use serde::{Serialize, Deserialize};
use std::fs;
use std::io::prelude::*;

mod cpu;

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
    #[serde(rename = "name")]
    Name: String,
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

    for data in &deserialized {
      // cpuにInitialをセット
      // cpuを1命令分動かす（？）
      // cpuの状態とFinalが合っているか確認
    }
}
