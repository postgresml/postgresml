import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = [
    "code",
    "result",
    "run",
    "history",
    "resultSection",
    "historySection",
  ]

  connect() {
    this.myCodeMirror = CodeMirror.fromTextArea(document.getElementById("codemirror-console"), {
      value: "SELECT 1\n",
      mode:  "sql",
      lineNumbers: true,
    });

    this.history = []
  }

  runQuery(event) {
    event.preventDefault()

    const query = event.currentTarget.querySelector("code").innerHTML

    this.myCodeMirror.setValue(query)
    this.run(event, query)
  }

  addQueryToHistory(query) {
    this.history.push(query)

    if (this.history.length > 10) {
      this.history.shift()
    }

    let innerHTML = ""

    // Templates? Please. React? Nah.
    for (let query of this.history.reverse()) {
      innerHTML += `
        <li >
          <a href="#query-results" data-action="click->console#runQuery">
            <span><code>${query}</code></span>
          </a>
        </li>
      `
    }

    this.historyTarget.innerHTML = innerHTML;
    this.historySectionTarget.classList.remove("hidden")
  }


  run(event, query) {
    this.runTarget.disabled = true
    this.resultSectionTarget.classList.remove("hidden")
    this.resultTarget.innerHTML = "Running..."

    if (!query) {
      query = this.myCodeMirror.getValue();
      this.addQueryToHistory(query)
    }

    myFetch(`/console/run/`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      redirect: "follow",
      body: JSON.stringify({
        "query": query,
      }),
    })
    .then(res => res.text())
    .then(html => {
      this.resultTarget.innerHTML = html
      this.runTarget.disabled = false
    })
  }
}
