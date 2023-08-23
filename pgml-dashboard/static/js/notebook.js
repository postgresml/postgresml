import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    'renameNotebookForm',
    'notebookName',
    'cell',
    'scroller',
  ];

  connect() {
    document.addEventListener('keyup', this.executeSelectedCell.bind(this))
    const rect = this.scrollerTarget.getBoundingClientRect()
    const innerHeight = window.innerHeight

    this.scrollerTarget.style.maxHeight = `${innerHeight - rect.top - 10}px`
  }

  disconnect() {
    document.removeEventListener('keyup', this.executeSelectedCell.bind(this))
  }

  renameNotebook(event) {
    this.renameNotebookFormTarget.classList.remove('hidden')
    this.notebookNameTarget.classList.add('hidden')
  }

  selectCell(event) {
    this.cellTargets.forEach(target => {
      target.classList.remove('selected')
    })

    if (event.currentTarget.classList.contains('sql') || event.currentTarget.querySelector('.notebook-cell-edit')) {
      event.currentTarget.classList.add('selected')
    }
  }

  executeSelectedCell(event) {
    if (event.shiftKey) {
      if (event.key === 'Enter' && event.keyCode === 13) {
        const selectedCellPlay = document.querySelector('.selected form[data-action="notebook-cell#play"]')

        if (selectedCellPlay) {
          selectedCellPlay.requestSubmit()
        }
      }
    }
  }
}
