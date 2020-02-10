import init, { compile } from "/pkg/lang_rs.js"

init().then(wasm => {
  console.log("initialised", wasm)

  const form = document.querySelector("form")

  form.addEventListener("submit", event => {
    event.preventDefault()

    const code = form.querySelector("textarea").value

    const result = compile(code)

    console.log("Compiled!", result)

    document.querySelector(".js-output").innerText = result
  })
})
