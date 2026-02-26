import * as fs from "node:fs";
import {
  DOMParser
} from "https://deno.land/x/deno_dom/deno-dom-wasm.ts";


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

    console.log(`
m.insert(0x${opcode}, OpCode::new(0x${opcode}, "${opname}", ${bytes}, ${clocks}, CycleCalcMode::None, AddressingMode::${addressingMode}));
`)
  }
}
