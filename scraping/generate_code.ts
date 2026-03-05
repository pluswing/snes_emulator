import * as fs from "node:fs";
import {
  DOMParser
} from "https://deno.land/x/deno_dom/deno-dom-wasm.ts";
import assert from "node:assert";

const ADDRESSING_MODE_TABLE: {[key: string]: string} = {
  "Program Counter Relative": "Program_Counter_Relative",
  "Implied (type 1)": "Implied",
  "Absolute": "Absolute",
  "Direct Page": "Direct_Page",
  "Direct Page Indexed by X": "Direct_Page_Indexed_by_X",
  "Stack (Push)": "Stack",
  "Stack (PC Relative Long)": "Stack",
  "Stack (Pull)": "Stack",
  "Stack (RTI)": "Stack",
  "Implied (type 2)": "Implied",
  "Immediate": "Immediate",
  "Absolute Long": "Absolute_Long",
  "Direct Page Indirect": "Direct_Page_Indirect",
  "Direct Page Indirect Long": "Direct_Page_Indirect_Long",
  "Absolute Indexed by X": "Absolute_Indexed_by_X",
  "Absolute Long Indexed by X": "Absolute_Long_Indexed_by_X",
  "Absolute Indexed by Y": "Absolute_Indexed_by_Y",
  "Direct Page Indexed Indirect by X": "Direct_Page_Indexed_Indirect_by_X",
  "Direct Page Indirect Indexed by Y": "Direct_Page_Indirect_Indexed_by_Y",
  "Direct Page Indirect Long Indexed by Y": "Direct_Page_Indirect_Long_Indexed_by_Y",
  "Stack Relative": "Stack_Relative",
  "Stack Relative Indirect Indexed by Y": "Stack_Relative_Indirect_Indexed_by_Y",
  "Direct Page Indexed by Y": "Direct_Page_Indexed_by_Y",
  "Absolute Indirect": "Absolute_Indirect",
  "Absolute Indexed Indirect": "Absolute_Indexed_Indirect",
  "Absolute Indirect Long": "Absolute_Indirect_Long",
  "Implied (type 3)": "Implied",
  "Accumulator": "Accumulator",
  "Stack (absolute)": "Stack",
  "Stack (RTL)": "Stack",
  "Block Move": "Block_Move",
  "Stack (Interrupt)": "Stack",
  "Program Counter Relative Long": "Program_Counter_Relative_Long",
  "Implied (type 3)[4]": "Implied",
  "Stack (RTS)": "Stack",
  "Stack (Direct Page Indirect)": "Stack"
}

const main = () => {
  outputHeader()
  outputOpsDefinition()
  outputCallFunction()
}

const outputHeader = () => {
  console.log(`
use std::{collections::HashMap};
use once_cell::sync::Lazy;
use crate::cpu::{AddressingMode, CPU, FLAG_MEMORY_ACCUMULATOR_MODE, MODE_16BIT, OpCode, OpInfo};
`.trim())
}

const outputOpsDefinition = () => {
  console.log("")
  console.log(`
pub static CPU_OPS_CODES: Lazy<HashMap<u8, OpCode>> = Lazy::new(|| {
  let mut m = HashMap::new();
`.trim())
  const files = fs.readdirSync("ops")
  const adressingModes = {}
  for (const file of files) {
    const opname = file.split(".")[0]
    const html = fs.readFileSync(`ops/${file}`).toString()
    const doc = new DOMParser().parseFromString(html, "text/html")
    const table = doc.querySelector(".wikitable")
    const tbodies = table?.querySelectorAll("tbody")
    for (let i = 0; i < tbodies!.length; i++) {
      if (i < 2) {
        continue;
      }
      const tbody = tbodies![i]
      const trs = tbody?.querySelectorAll("tr")!
      for (const tr of trs) {
        const tds = tr.querySelectorAll("td")
        const addressingMode = tds[0].innerText.trim()
        const opcode = tds[1].innerText.trim()
        const bytes = tds[2].innerText.trim()
        const clocks = tds[3].innerText.trim()
        const [emulationBytes, nativeBytes] = parseBytes(bytes)
        const [emulationClocks, nativeClocks] = parseClocks(clocks)
        const am = ADDRESSING_MODE_TABLE[addressingMode]
        assert(am)
        if (!adressingModes[addressingMode]) {
          adressingModes[addressingMode] = {}
        }
        adressingModes[addressingMode][opname] = true;

        console.log(`  m.insert(0x${opcode}, OpCode::new(0x${opcode}, "${opname}", OpInfo::new(${nativeBytes}, ${nativeClocks}), OpInfo::new(${emulationBytes}, ${emulationClocks}), AddressingMode::${am}));`)
      }
    }
  }
  console.log(`
  m
});`.trim())

  console.log("/*")
  console.log(JSON.stringify(adressingModes, null, 2))
  console.log("*/")
}

const parseBytes = (rawBytes: string): [number, number] => {
  // "2/3 bytes"
  // "3 bytes"
  assert(rawBytes.match(/^[1-9](\/[1-9])? bytes?$/), rawBytes)
  const nums = rawBytes.replace(" bytes", "").replace(" byte", "")
  if (nums.includes("/")) {
    const [e, n] = nums.split("/")
    return [parseInt(e, 10), parseInt(n, 10)]
  } else {
    return [parseInt(nums), parseInt(nums)]
  }
}

const parseClocks = (rawCycles: string): [number, number] => {
  // "2 cycles*"
  // 7 cycles per byte moved
  rawCycles = rawCycles.replace(" per byte moved", "")
  assert(rawCycles.match(/^[1-9] cycles\*?$/), rawCycles)
  const nums = rawCycles.replace(" cycles", "").replace("*", "")
  return [parseInt(nums), parseInt(nums)]
}

const outputCallFunction = () => {
  console.log("")
  console.log(`
pub fn call(cpu: &mut CPU, op: &OpCode) {
  match op.name.as_str() {
    `)

  const files = fs.readdirSync("ops")
  for (const file of files) {
    const opname = file.split(".")[0]
    console.log(`
  "${opname}" => {
      cpu.${opname.toLowerCase()}(&op.addressing_mode);
      let bytes = if cpu.is_accumulator_16bit_mode() {
        op.native.bytes
      } else {
        op.emulation.bytes
      };
      cpu.program_counter = cpu.program_counter.wrapping_add(bytes - 1);
    }
`)
  }
    console.log(`
  &_ => {
    todo!("OP: {} not defined!", op.name);
  }
  }
}
`)
}

main()


