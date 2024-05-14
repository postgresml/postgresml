import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    'cell',
    'scroller',
    'cellButton',
    'stopButton',
    'playAllButton',
    'newCell',
    'syntaxName',
    'playButtonText',
  ];

  static outlets = ['modal'];

  static values = {
    urlRoot: String,
  }

  cellCheckIntervalMillis = 500

  connect() {
    document.addEventListener('keyup', this.executeSelectedCell.bind(this))
    const rect = this.scrollerTarget.getBoundingClientRect()
    const innerHeight = window.innerHeight

    this.scrollerTarget.style.maxHeight = `${innerHeight - rect.top - 10}px`
    // this.confirmDeleteModal = new bootstrap.Modal(this.deleteModalTarget)

    this.sortable = Sortable.create(this.scrollerTarget, {
      onUpdate: this.updateCellOrder.bind(this),
      onStart: this.makeCellsReadOnly.bind(this),
      onEnd: this.makeCellsEditable.bind(this),
    })
  }

  disconnect() {
    document.removeEventListener('keyup', this.executeSelectedCell.bind(this))
  }

  makeCellsReadOnly(event) {
    this.codeMirrorReadOnly(true)
  }

  makeCellsEditable(event) {
    this.codeMirrorReadOnly(false)
  }

  codeMirrorReadOnly(readOnly) {
    const cells = document.querySelectorAll(`div[data-cell-id]`)

    cells.forEach(cell => {
      const controller = this.application.getControllerForElementAndIdentifier(cell, 'notebook-cell')
      if (controller.codeMirror) {
        controller.codeMirror.setOption('readOnly', readOnly)
      }
    })
  }

  updateCellOrder(event) {
    const cells = [...this.scrollerTarget.querySelectorAll('turbo-frame')]
    const notebookId = this.scrollerTarget.dataset.notebookId
    const ids = cells.map(cell => parseInt(cell.dataset.cellId))

    fetch(`${this.urlRootValue}/${notebookId}/reorder`, {
      method: 'POST',
      body: JSON.stringify({
        cells: ids,
      }),
      headers: {
        'Content-Type': 'application/json',
      }
    })

    this.scrollerTarget.querySelectorAll('div[data-cell-number]').forEach((cell, index) => {
      cell.dataset.cellNumber = index + 1
      cell.innerHTML = index + 1
    })
  }

  playAll(event) {
    event.currentTarget.disabled = true
    const frames = this.scrollerTarget.querySelectorAll('turbo-frame[data-cell-type="3"]')
    this.playCells([...frames])
  }

  playCells(frames) {
    const frame = frames.shift()
    const form = document.querySelector(`form[data-cell-play-id="${frame.dataset.cellId}"]`)
    const cellType = form.querySelector('input[name="cell_type"]').value
    const contents = form.querySelector('textarea[name="contents"]').value
    const body = `cell_type=${cellType}&contents=${encodeURIComponent(contents)}`

    fetch(form.action, {
      method: 'POST',
      body,
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8',
      },
    }).then(response => {
      // Reload turbo frame
      frame.querySelector('a[data-notebook-target="loadCell"]').click()

      if (response.status > 300) {
        throw new Error(response.statusText)
      }

      if (frames.length > 0) {
        setTimeout(() => this.playCells(frames), 250)
      } else {
        this.playAllButtonTarget.disabled = false
      }
    })
  }

  // Check that the cell finished running.
  // We poll the DOM every 500ms. Not a very clever solution, but I think it'll work.
  checkCellState() {
    const cell = document.querySelector(`div[data-cell-id="${this.activeCellId}"]`)

    if (cell.dataset.cellState === 'rendered') {
      this.playButtonTextTarget.innerHTML = 'Run'
      clearInterval(this.cellCheckInterval)
      this.enableCellButtons()
      this.stopButtonTarget.disabled = true
    }
  }

  playCell(event) {
    // Start execution.
    const cell = document.querySelector(`div[data-cell-id="${this.activeCellId}"]`)

    const form = cell.querySelector(`form[data-cell-play-id="${this.activeCellId}"]`)
    form.requestSubmit()
    
    if (cell.dataset.cellType === '3') {
      this.playButtonTextTarget.innerHTML = 'Running'
      this.disableCellButtons()
      
      cell.dataset.cellState = 'running'

      // Check on the result of the cell every 500ms.
      this.cellCheckInterval = setInterval(this.checkCellState.bind(this), this.cellCheckIntervalMillis)

      // Enable the stop button if we're running code.
      this.stopButtonTarget.disabled = false
    }
  }

  playStop() {
    this.stopButtonTarget.disabled = true
    this.disableCellButtons()

    const form = document.querySelector(`form[data-cell-stop-id="${this.activeCellId}"]`)
    form.requestSubmit()

    // The query will be terminated immediately, unless there is a real problem.
    this.enableCellButtons()
  }

  enableCellButtons() {
    this.cellButtonTargets.forEach(target => target.disabled = false)
  }

  disableCellButtons() {
    this.cellButtonTargets.forEach(target => target.disabled = true)
  }

  selectCell(event) {
    if (event.currentTarget.classList.contains('active')) {
      return
    }

    this.enableCellButtons()
    this.activeCellId = event.currentTarget.dataset.cellId

    this.cellTargets.forEach(target => {
      if (target.classList.contains('active')) {
        // Reload the cell from the backend, i.e. cancel the edit.
        target.querySelector('a[data-notebook-target="loadCell"]').click()
      }
    })

    if (!event.currentTarget.classList.contains('active')) {
      event.currentTarget.classList.add('active')
    }

    let cellType = 'SQL'
    if (event.currentTarget.dataset.cellType === '1') {
      cellType = 'Markdown'
    }

    this.syntaxNameTarget.innerHTML = cellType
  }

  executeSelectedCell(event) {
    if (!this.activeCellId) {
      return
    }

    if (event.shiftKey) {
      if (event.key === 'Enter' && event.keyCode === 13) {
        this.playCell()
      }
    }
  }

  deleteCellConfirm() {
    this.modalOutlet.show()
  }

  deleteCell() {
    const form = document.querySelector(`form[data-cell-delete-id="${this.activeCellId}"]`)
    form.requestSubmit()
  }

  newCell() {
    this.newCellTarget.requestSubmit()
  }

  changeSyntax(event) {
    event.preventDefault()
    const syntax = event.currentTarget.dataset.syntax

    const cell = document.querySelector(`div[data-cell-id="${this.activeCellId}"]`)
    const controller = this.application.getControllerForElementAndIdentifier(cell, 'notebook-cell')
    controller.setSyntax(event.currentTarget.dataset.syntax)

    if (syntax === 'gfm') {
      this.syntaxNameTarget.innerHTML = 'Markdown'
    } else {
      this.syntaxNameTarget.innerHTML = 'SQL'
    }
  }
}
