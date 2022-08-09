import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    "newLineCode",
  ];

  connect() {
    this.newLineCodeMirror = CodeMirror.fromTextArea(this.newLineCodeTarget, {
      lineWrapping: true,
      matchBrackets: true,
    })

    this.newLineCodeMirror.setSize('100%', 100)
    window.onload = function() {
      window.scrollTo(0, document.body.scrollHeight)
    }
  }
}
