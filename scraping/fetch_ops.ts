import {
  DOMParser
} from "https://deno.land/x/deno_dom/deno-dom-wasm.ts";

const res = await fetch("https://sneslab.net/wiki/65c816_Opcode_Matrix")
const html = await res.text()
const doc = new DOMParser().parseFromString(html, "text/html")
const table = doc.querySelector(".wikitable")
const links = table.querySelectorAll("td > a")
const details: string[] = []
links.forEach((l) => details.push(l.getAttribute("href")))
const uniq = [...new Set(details)]
console.log(uniq)

const encoder = new TextEncoder();
for (const path of uniq) {
  console.log(`fetch ${path}`)
  const url = `https://sneslab.net${path}`
  const op = path.split("/")[2]
  const res = await fetch(url)
  const html = await res.text()
  Deno.writeFileSync(`ops/${op}.html`, encoder.encode(html))
}
