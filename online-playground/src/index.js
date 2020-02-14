import init from "/online_playground.js"

const wabt = window.WabtModule()

const raf = window.requestAnimationFrame

const output = document.getElementById("output")
const logs = document.getElementById("logs")
let prev_wat = ""

async function log_output() {
  raf(log_output)

  const wat = output.innerText

  if (wat === prev_wat) return

  prev_wat = wat

  try {
    console.log("wabt", wabt)
    console.log("wat", wat)
    const module = wabt.parseWat("", wat, {})
    const { buffer } = module.toBinary({})

    module.destroy()

    const wasm_mod = await WebAssembly.instantiate(buffer)
    console.log("wasm_mod", wasm_mod)

    logs.innerText = wasm_mod.instance.exports.main()
  } catch (e) {
    logs.innerText = `⚠️ Error running program: ${e}`
    console.log("Run program error", e)
  }
}

init().then(wasm => {
  console.log("initialised", wasm)

  raf(log_output)
})
