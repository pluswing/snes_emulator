import * as fs from "node:fs";
import {
  DOMParser
} from "https://deno.land/x/deno_dom/deno-dom-wasm.ts";
import assert from "node:assert";

const ADDRESSING_MODE_TABLE: {[key: string]: string} = {
  "Program Counter Relative": "",
  "Implied (type 1)": "Implied",
  "Absolute": "Absolute",
  "Stack (Push)": "",
  "Stack (PC Relative Long)": "",
  "Stack (Pull)": "",
  "Stack (RTI)": "",
  "Implied (type 2)": "",
  "Immediate": "",
  "Implied (type 3)": "",
  "Accumulator": "Accumulator",
  "Stack (absolute)": "",
  "Stack (RTL)": "",
  "Block Move": "",
  "Stack (Interrupt)": "",
  "Program Counter Relative Long": "Program_Counter_Relative_Long",
  "Implied (type 3)[4]": "",
  "Stack (RTS)": "",
  "Stack (Direct Page Indirect)": "",
}

const addressingModes: string[] = []
const main = () => {
  const files = fs.readdirSync("ops")
  for (const file of files) {
    const opname = file.split(".")[0]
    const html = fs.readFileSync(`ops/${file}`).toString()
    const doc = new DOMParser().parseFromString(html, "text/html")
    const table = doc.querySelector(".wikitable")
    const tbody = table?.querySelectorAll("tbody")[2]
    const trs = tbody?.querySelectorAll("tr")!
    for (const tr of trs) {
      const tds = tr.querySelectorAll("td")
      const addressingMode = tds[0].innerText.trim()
      const opcode = tds[1].innerText.trim()
      const bytes = tds[2].innerText.trim()
      const clocks = tds[3].innerText.trim()
      console.log({
        addressingMode,
        opcode,
        bytes,
        clocks
      })
      const [emulationBytes, nativeBytes] = parseBytes(bytes)
      const [emulationClocks, nativeClocks] = parseClocks(clocks)

      console.log(`
  m.insert(0x${opcode}, OpCode::new(0x${opcode}, "${opname}", OpInfo::new(${nativeBytes}, ${nativeClocks}), OpInfo::new(${emulationBytes}, ${emulationClocks}), AddressingMode::${ADDRESSING_MODE_TABLE[addressingMode]}));
  `)

      addressingModes.push(addressingMode)
    }
  }
  console.log([...new Set(addressingModes)])
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

main()


