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
  // outputHeader()
  // outputOpsDefinition()
  // outputCallFunction()
  const defs = parse()
  // console.log(defs);
  outputCyclesFunction(defs)
  outputCallFunction(defs)
}

type Definision = {
  name: string,
  am: string,
  op: string,
  bytes: string,
  cycles: string,
  codes: string[],
}

const parse = (): Definision[] => {
  const files = fs.readdirSync("ops")
  let allDefs = []
  for (const file of files) {
    const opname = file.split(".")[0]
    const html = fs.readFileSync(`ops/${file}`).toString()
    const doc = new DOMParser().parseFromString(html, "text/html")

    // Immediate	E8	2 bytes	3 cycles
    const trs = doc.querySelectorAll(".wikitable > tbody > tr")
    const ops = Array.from(trs).map(tr => {
      return tr.querySelectorAll("td")
    }).filter((tds) => {
      return tds.length == 4 && tds[0].innerText.trim() != "Addressing Mode"
    })

    // MOV A, #imm
    const pre = doc.querySelector("pre")
    const texts = pre.innerText.split("\n")
      .map(t => t.trim())
      .filter(t => t.startsWith(opname))

    const defs = ops.map((tds, i) => {
      const cyclesText = tds[3].innerText.trim().replace(/\n/g, "").replace(/.*(\d+) cycles.*/, "$1")

      return {
        name: opname,
        am: tds[0].innerText.trim(),
        op: tds[1].innerText.trim(),
        bytes: tds[2].innerText.replace(/bytes?/, "").trim(),
        cycles: cyclesText,
        codes: texts[texts.length > i ? i : 0].split(/[ ,]/).map(t => t.trim()).filter(t => t)
        }
    })
    allDefs = [...allDefs, ...defs]
  }
  return allDefs
}

const outputCyclesFunction = (defs: Definision[]) => {
  console.log("fn get_cycles(op: u8) -> u8 {")
  console.log("  match op {")
  defs.forEach(d => {
    console.log(`    0x${d.op} => ${d.cycles},`)
  })
  console.log("  }")
  console.log("}")
}

const outputCallFunction = (defs: Definision[]) => {
  console.log("fn call(cpu: &mut APU, op: u8) {")
  console.log("  match op {")
  defs.forEach(d => {
    console.log(`    0x${d.op} => cpu.${d.name.toLowerCase()}(),`)
  })
  console.log("  }")
  console.log("}")
}

// const outputHeader = () => {
//   console.log(`
// use std::{collections::HashMap};
// use once_cell::sync::Lazy;
// use crate::cpu::{AddressingMode, CPU, FLAG_MEMORY_ACCUMULATOR_MODE, MODE_16BIT, OpCode, OpInfo};
// `.trim())
// }
/*
const outputCallFunction = (defs: Definision[]) => {
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
*/

main()


