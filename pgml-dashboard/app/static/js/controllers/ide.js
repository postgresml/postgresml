import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = [
    "code",
    "result",
    "run",
  ]

  connect() {
    this.myCodeMirror = CodeMirror.fromTextArea(document.getElementById("codemirror-ide"), {
      value: "SELECT 1\n",
      mode:  "sql",
      lineNumbers: true,
    });
  }


  run(event) {
    this.runTarget.disabled = true
    this.resultTarget.classList.remove("hidden")
    this.resultTarget.innerHTML = 'Running...'

    fetch(`/ide/run/`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      redirect: "follow",
      body: JSON.stringify({
        "query": this.myCodeMirror.getValue(),
      }),
    })
    .then(res => res.text())
    .then(html => {
      this.resultTarget.innerHTML = html
      this.runTarget.disabled = false
      
    })
  }
}
