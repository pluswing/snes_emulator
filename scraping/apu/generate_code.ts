import * as fs from "node:fs";
import {
  DOMParser
} from "https://deno.land/x/deno_dom/deno-dom-wasm.ts";
import assert from "node:assert";

const ARGS_TABLE: {[key: string]: string} = {
  "A": "&Args::RegisterA",
  "#imm": "&Args::Immediate",
  "(X)": "&Args::RegisterX_Direct_Page_Indirect",
  "(X)+": "&Args::RegisterX_Direct_Page_Indirect_Incriment",
  "dp": "&Args::DirectPage",
  "dp+X": "&Args::Direct_Page_Indexed_by_X",
  "!abs": "&Args::Absolute",
  "!abs+X": "&Args::Absolute_Indexed_by_X",
  "!abs+Y": "&Args::Absolute_Indexed_by_Y",
  "[dp+X]": "&Args::Direct_Page_Indexed_Indirect_by_X",
  "[dp]+Y": "&Args::Direct_Page_Indirect_Indexed_by_Y",
  "X": "&Args::RegisterX",
  "dp+Y": "&Args::Direct_Page_Indexed_by_Y",
  "Y": "&Args::RegisterY",
  "SP": "&Args::StackPointer",
  // 特殊パターン
  "dp<d>": "&Args::DirectPage",
  "dp<s>": "&Args::DirectPage",
  "dp bit": "op",
  "dip. bit": "op",
  "YA": "&Args::RegisterYA",
  "PSW": "ProgramStatus",
  "C": "/* C */",
  "mem.": "/* mem. */",
  "/mem.": "/* /mem. */",
  "(Y)": "/* (Y) */",
  "[!abs+X]": "/* [!abs+X] */",
  "dp+x": "/* dp+x */",
  "mem. bit": "op",
  "upage": "/* upage */",
  "bit": "/* bit */"
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
      let codes = texts[texts.length > i ? i : 0].split(/[ ,]/).map(t => t.trim()).filter(t => t).slice(1).filter(t => t != "rel");
      if (codes[1] == "bit") {
        codes = [`${codes[0]} ${codes[1]}`, ...codes.slice(1)]
      }
      return {
        name: opname,
        am: tds[0].innerText.trim(),
        op: tds[1].innerText.trim(),
        bytes: tds[2].innerText.replace(/bytes?/, "").trim(),
        cycles: cyclesText,
        codes
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
  // defs = defs.filter(d => d.name == "MOV")
  const args = [...new Set(defs.map(d => d.codes).flat())]
  const undef = args.filter(a => !ARGS_TABLE[a])
  if (undef.length) {
    console.log(`UNDEF: ${undef}`)
    return
  }
  console.log("fn call(cpu: &mut APU, op: u8) {")
  console.log("  match op {")
  defs.forEach(d => {
    const args = d.codes.map(c => ARGS_TABLE[c])
    console.log(`    0x${d.op} => cpu.${d.name.toLowerCase()}(${args.join(", ")}),`)
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


