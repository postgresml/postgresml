import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    'renameNotebookForm',
    'notebookName',
  ];

  renameNotebook(event) {
    this.renameNotebookFormTarget.classList.remove('hidden')
    this.notebookNameTarget.classList.add('hidden')
  }
}
