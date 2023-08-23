import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    'cell',
    'scroller',
    'cellButton',
    'loadCell',
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

  playAll() {
    const forms = document.querySelectorAll('form[data-form-role="play"]')
    forms.forEach(form => form.requestSubmit())
  }

  playCell() {
    const form = document.querySelector(`form[data-cell-play-id="${this.activeCellId}"]`)
    form.requestSubmit()
  }

  enableCellButtons() {
    this.cellButtonTargets.forEach(target => {
      target.disabled = false
    })
  }

  renameNotebook(event) {
    this.renameNotebookFormTarget.classList.remove('hidden')
    this.notebookNameTarget.classList.add('hidden')
  }

  selectCell(event) {
    if (event.currentTarget.classList.contains('active')) {
      return
    }

    this.enableCellButtons()
    this.activeCellId = event.currentTarget.dataset.cellId

    this.cellTargets.forEach(target => {
      if (target.classList.contains('active')) {
        target.querySelector('a[data-notebook-target="loadCell"]').click()
      }
    })

    if (!event.currentTarget.classList.contains('active')) {
      event.currentTarget.classList.add('active')
    }

    // if (event.currentTarget.classList.contains('sql') || event.currentTarget.querySelector('.notebook-cell-edit')) {
    //   event.currentTarget.classList.add('selected')
    // }
  }

  executeSelectedCell(event) {
    if (event.shiftKey) {
      if (event.key === 'Enter' && event.keyCode === 13) {
        const selectedCellPlay = document.querySelector('.selected form[data-form-role="play"]')

        if (selectedCellPlay) {
          selectedCellPlay.requestSubmit()
        }
      }
    }
  }
}
