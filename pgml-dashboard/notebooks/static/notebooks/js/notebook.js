import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    'newLineCode',
    'lines',
  ];

  connect() {
    this.newLineCodeMirror = CodeMirror.fromTextArea(this.newLineCodeTarget, {
      lineWrapping: true,
      matchBrackets: true,
    })

    this.newLineCodeMirror.setSize('100%', 100)
  }

  deleteLine(event) {
    event.preventDefault()
    const target = event.currentTarget

    const form = new FormData(target)
    const url = target.action
    const body = new URLSearchParams(form)

    fetch(url, {
      method: 'POST',
      body,
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8',
      },
    })
    .then(res => res.text())
    .then(_ => {
      const id = target.dataset.lineId
      document.getElementById(`line-${id}`).remove()
    })
  }

  // Add a new line to the notebook.
  // Submit via AJAX, add the result to the end of the notebook,
  // and scroll to the bottom of the page.
  addLine(event) {
    event.preventDefault()
    const target = event.currentTarget
    
    const form = new FormData(target)
    const url = target.action
    const entries = []

    for (let entry of form.entries()) {
      if (entry[0] === 'csrfmiddlewaretoken')
        entries.push(`${entry[0]}=${entry[1]}`)
    }

    entries.push(`contents=${encodeURIComponent(this.newLineCodeMirror.getValue())}`)

    fetch(url, {
      method: 'POST',
      body: entries.join('&'),
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8',
        'X-CSRFToken': entries['csrfmiddlewaretoken'],
      },
    })
    .then(res => res.text())
    .then(text => {
      this.linesTarget.innerHTML += text
      window.scrollTo(0, document.body.scrollHeight)
      this.newLineCodeMirror.setValue('')
    })
  }
}
