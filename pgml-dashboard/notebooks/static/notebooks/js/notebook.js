import { Controller } from '@hotwired/stimulus'
import hljs from  '@highlight'

export default class extends Controller {
  static targets = [
    'newCellCode',
    'cells',
    'existingCell',
    'renameNotebookForm',
    'notebookName',
    'newCellForm',
  ];

  connect() {
    this.newCellCodeMirror = CodeMirror.fromTextArea(this.newCellCodeTarget, {
      lineWrapping: true,
      matchBrackets: true,
      mode: 'gfm', // Github markdown
      scrollbarStyle: 'null',
    })

    this.newCellCodeMirror.setSize('100%', 250)
    this.newCellCodeMirror.on('change', () => this.detectSql(this.newCellCodeMirror))

    const keyMap = {
      'Ctrl-Enter': () => this.newCellFormTarget.requestSubmit(),
      'Cmd-Enter': () => this.newCellFormTarget.requestSubmit(),
      'Ctrl-/': () => this.newCellCodeMirror.execCommand('toggleComment'),
      'Cmd-/': () => this.newCellCodeMirror.execCommand('toggleComment'),
    };

    this.newCellCodeMirror.addKeyMap(keyMap)

    this.exitingCellsCodeMirror = {}
    this.deleteTimeouts = {}

    // Highlight all existing code segments
    // document.querySelectorAll('.language-sql').forEach(target => hljs.highlightElement(target))
  }

  initCodeMirrorOnTarget(target) {
    const cellId = target.dataset.cellId

    const codeMirror = CodeMirror.fromTextArea(target, {
      lineWrapping: true,
      matchBrackets: true,
      mode: 'gfm',
      scrollbarStyle: 'null',
    })

    codeMirror.setSize('100%', 250)
    codeMirror.on('change', () => this.detectSql(codeMirror))

    const keyMap = {
      'Ctrl-Enter': () => document.getElementById(`edit-cell-form-${cellId}`).requestSubmit(),
      'Cmd-Enter': () => document.getElementById(`edit-cell-form-${cellId}`).requestSubmit(),
      'Ctrl-/': () => codeMirror.execCommand('toggleComment'),
      'Cmd-/': () => codeMirror.execCommand('toggleComment'),
    };

    codeMirror.addKeyMap(keyMap)

    // Has value already?
    this.detectSql(codeMirror)

    this.exitingCellsCodeMirror[target.dataset.cellId] = codeMirror
  }

  renameNotebook(event) {
    this.renameNotebookFormTarget.classList.remove('hidden')
    this.notebookNameTarget.classList.add('hidden')
  }

  detectSql(codeMirror) {
    const value = codeMirror.getValue()

    if (value.startsWith('%%sql'))
      codeMirror.setOption('mode', 'sql')
    else
      codeMirror.setOption('mode', 'gfm')
  }

  enableEdit(event) {
    const target = event.currentTarget
    const cellId = target.dataset.cellId

    const cell = document.getElementById(`cell-${cellId}`)

    // Display the textarea first to get proper dimensions into the DOM
    cell.querySelector('.notebook-cell-edit').classList.remove('hidden')

    // Initialize CodeMirror after element is rendered
    this.initCodeMirrorOnTarget(cell.querySelector('.notebook-cell-edit textarea'))

    target.remove()
  }

  editCell(event) {
    event.preventDefault()
    const target = event.currentTarget
    const cellId = target.dataset.cellId

    const button = target.querySelector('button[type=submit]')
    button.disabled = true
    button.querySelector('span').innerHTML = 'pending'

    this.exitingCellsCodeMirror[cellId].save()

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
      const child = document.getElementById(`cell-${cellId}`)

      // Build new cell element
      const template = document.createElement('template')
      text = text.trim()
      template.innerHTML = text


      const newChild = template.content.firstChild
      const newCellId = newChild.dataset.cellId

      // Replace old cell with new cell
      child.parentNode.replaceChild(newChild, child)

      // const codeElement = newChild.querySelector('.language-sql')

      // if (codeElement) {
      //   hljs.highlightElement(codeElement)
      // }

      // Don't leak memory
      delete this.exitingCellsCodeMirror[cellId]
    })
  }

  deleteCell(event) {
    event.preventDefault()
    const target = event.currentTarget

    const form = new FormData(target)
    const url = target.action
    const body = new URLSearchParams(form)
    const cellId = target.dataset.cellId

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

      const child = document.getElementById(`cell-${cellId}`)
      child.parentNode.replaceChild(template.content.firstChild, child)

      // Remove the undo in 5 seconds
      this.deleteTimeouts[cellId] = window.setTimeout(() => {
        document.getElementById(`cell-${cellId}`).remove()
        delete this.deleteTimeouts[cellId]
      }, 5000)
    })
  }

  undoDeleteCell(event) {
    event.preventDefault()

    const target = event.currentTarget
    const cellId = target.dataset.cellId

    window.clearTimeout(this.deleteTimeouts[cellId])
    delete this.deleteTimeouts[cellId]

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

      const child = document.getElementById(`cell-${cellId}`)
      child.parentNode.replaceChild(template.content.firstChild, child)
    })
  }

  // Add a new cell to the notebook.
  // Submit via AJAX, add the result to the end of the notebook,
  // and scroll to the bottom of the page.
  addCell(event) {
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

    entries.push(`contents=${encodeURIComponent(this.newCellCodeMirror.getValue())}`)

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
      this.cellsTarget.innerHTML += text
      window.scrollTo(0, document.body.scrollHeight)
      this.newCellCodeMirror.setValue('')

      // Re-enable the submit button
      button.disabled = false
      button.querySelector('span').innerHTML = 'play_arrow'

      // const highlightTarget = document.querySelector('.language-sql:not(.hljs)')

      // if (highlightTarget)
      //   hljs.highlightElement(highlightTarget)
    })
  }
}
