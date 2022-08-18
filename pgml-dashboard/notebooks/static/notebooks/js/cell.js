import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    "self",
    "editor",
    "form",
    'undo',
  ];

  connect() {
    // Enable CodeMirror editor if we are editing.
    if (this.hasEditorTarget && this.hasFormTarget) {
      this.initCodeMirrorOnTarget(this.editorTarget)
    }

    if (this.hasUndoTarget) {
      setTimeout(function() {
        // I think undoTarget is preserved after reload, not sure
        if (this.hasUndoTarget && this.undoTarget.parentNode) {
          this.undoTarget.parentNode.remove()
        }
      }.bind(this), 5000)
    }
  }

  // Enable CodeMirror on target.
  initCodeMirrorOnTarget(target) {
    this.codeMirror = CodeMirror.fromTextArea(target, {
      lineWrapping: true,
      matchBrackets: true,
      mode: 'sql',
      scrollbarStyle: 'null',
    })

    this.codeMirror.setSize('100%', 250)

    const keyMap = {
      'Ctrl-Enter': () => this.formTarget.requestSubmit(),
      'Cmd-Enter': () => this.formTarget.requestSubmit(),
      'Ctrl-/': () => this.codeMirror.execCommand('toggleComment'),
      'Cmd-/': () => this.codeMirror.execCommand('toggleComment'),
    };

    this.codeMirror.addKeyMap(keyMap)
  }

  // Change syntax highlighting.
  selectCellType(event) {
    const value = event.target.options[event.target.selectedIndex].value

    if (value == 3) {
      this.codeMirror.setOption('mode', 'sql')
    } else {
      this.codeMirror.setOption('mode', 'gfm')
    }
  }

  freezeScrollOnNextRender() {
    document.addEventListener("turbo:render", scrollToBottom);
  }
}

const scrollToBottom = () => {
  window.Turbo.navigator.currentVisit.scrolled = true;
  window.scrollTo(0, document.body.scrollHeight)
  document.removeEventListener("turbo:render", scrollToBottom);
};
