import { Controller } from '@hotwired/stimulus'
import hljs from  '@highlight'

export default class extends Controller {
  static targets = [
    'newLineCode',
    'lines',
    'existingLine',
  ];

  connect() {
    this.newLineCodeMirror = CodeMirror.fromTextArea(this.newLineCodeTarget, {
      lineWrapping: true,
      matchBrackets: true,
      mode: "gfm", // Github markdown
    })

    this.newLineCodeMirror.setSize('100%', 100)
    this.newLineCodeMirror.on("change", () => this.detectSql(this.newLineCodeMirror))

    this.exitingLinesCodeMirror = {}
    this.deleteTimeouts = {}

    // Highlight all existing code segments
    // document.querySelectorAll('.language-sql').forEach(target => hljs.highlightElement(target))
  }

  initCodeMirrorOnTarget(target) {
    const codeMirror = CodeMirror.fromTextArea(target, {
      lineWrapping: true,
      matchBrackets: true,
      mode: "gfm",
    })

    codeMirror.setSize('100%', 100)
    codeMirror.on("change", () => this.detectSql(codeMirror))

    // Has value already?
    this.detectSql(codeMirror)

    this.exitingLinesCodeMirror[target.dataset.lineId] = codeMirror
  }

  detectSql(codeMirror) {
    const value = codeMirror.getValue()

    if (value.startsWith('%%sql'))
      codeMirror.setOption("mode", "sql")
    else
      codeMirror.setOption("mode", "gfm")
  }

  enableEdit(event) {
    const target = event.currentTarget
    const lineId = target.dataset.lineId

    const line = document.getElementById(`line-${lineId}`)

    // Display the textarea first to get proper dimensions into the DOM
    line.querySelector('.notebook-line-edit').classList.remove('hidden')

    // Initialize CodeMirror after element is rendered
    this.initCodeMirrorOnTarget(line.querySelector('.notebook-line-edit textarea'))

    target.remove()
  }

  editLine(event) {
    event.preventDefault()
    const target = event.currentTarget
    const lineId = target.dataset.lineId

    const button = target.querySelector('button[type=submit]')
    button.disabled = true
    button.querySelector('span').innerHTML = 'pending'

    this.exitingLinesCodeMirror[lineId].save()

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
    .then(text => {
      const child = document.getElementById(`line-${lineId}`)

      // Build new line element
      const template = document.createElement('template')
      text = text.trim()
      template.innerHTML = text


      const newChild = template.content.firstChild
      const newLineId = newChild.dataset.lineId

      // Replace old line with new line
      child.parentNode.replaceChild(newChild, child)

      // const codeElement = newChild.querySelector('.language-sql')

      // if (codeElement) {
      //   hljs.highlightElement(codeElement)
      // }

      // Don't leak memory
      delete this.exitingLinesCodeMirror[lineId]
    })
  }

  deleteLine(event) {
    event.preventDefault()
    const target = event.currentTarget

    const form = new FormData(target)
    const url = target.action
    const body = new URLSearchParams(form)
    const lineId = target.dataset.lineId

    fetch(url, {
      method: 'POST',
      body,
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8',
      },
    })
    .then(res => res.text())
    .then(text => {
      const template = document.createElement('template')
      text = text.trim()
      template.innerHTML = text

      const child = document.getElementById(`line-${lineId}`)
      child.parentNode.replaceChild(template.content.firstChild, child)

      // Remove the undo in 5 seconds
      this.deleteTimeouts[lineId] = window.setTimeout(() => {
        document.getElementById(`line-${lineId}`).remove()
        delete this.deleteTimeouts[lineId]
      }, 5000)
    })
  }

  undoDeleteLine(event) {
    event.preventDefault()

    const target = event.currentTarget
    const lineId = target.dataset.lineId

    window.clearTimeout(this.deleteTimeouts[lineId])
    delete this.deleteTimeouts[lineId]

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
    .then(text => {
      const template = document.createElement('template')
      text = text.trim()
      template.innerHTML = text

      const child = document.getElementById(`line-${lineId}`)
      child.parentNode.replaceChild(template.content.firstChild, child)
    })
  }

  // Add a new line to the notebook.
  // Submit via AJAX, add the result to the end of the notebook,
  // and scroll to the bottom of the page.
  addLine(event) {
    event.preventDefault()
    const target = event.currentTarget

    const button = target.querySelector('button[type=submit]')
    button.querySelector('span').innerHTML = 'pending'

    // Disable button so people don't double push on slow queries
    button.disabled = true
    
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

      // Re-enable the submit button
      button.disabled = false
      button.querySelector('span').innerHTML = 'play_arrow'

      // const highlightTarget = document.querySelector('.language-sql:not(.hljs)')

      // if (highlightTarget)
      //   hljs.highlightElement(highlightTarget)
    })
  }
}
