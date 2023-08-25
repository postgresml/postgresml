import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    'editor',
    'form',
    'undo',
    'play',
    'type',
    'cancelEdit',
    'cell',
    'cellType',
    'dragAndDrop',
    'running',
    'executionTime',
  ];

  connect() {
    // Enable CodeMirror editor if we are editing.
    if (this.hasEditorTarget && !this.codeMirror) {
      this.initCodeMirrorOnTarget(this.editorTarget)
    }

    if (this.cellTarget.dataset.cellState === 'new') {
      this.cellTarget.scrollIntoView()
    }

    this.cellTarget.addEventListener('mouseover', this.showDragAndDrop.bind(this))
    this.cellTarget.addEventListener('mouseout', this.hideDragAndDrop.bind(this))
  }

  showDragAndDrop(event) {
    this.dragAndDropTarget.classList.remove('d-none')
  }

  hideDragAndDrop(event) {
    this.dragAndDropTarget.classList.add('d-none')
  }

  // Enable CodeMirror on target.
  initCodeMirrorOnTarget(target) {
    let mode = 'sql'

    if (target.dataset.type === 'markdown') {
      mode = 'gfm'
    }

    this.codeMirror = CodeMirror.fromTextArea(target, {
      lineWrapping: true,
      matchBrackets: true,
      mode,
      scrollbarStyle: 'null',
      lineNumbers: mode === 'sql',
    })

    this.codeMirror.setSize('100%', 'auto')

    const keyMap = {
      'Ctrl-Enter': () => this.formTarget.requestSubmit(),
      'Cmd-Enter': () => this.formTarget.requestSubmit(),
      'Ctrl-/': () => this.codeMirror.execCommand('toggleComment'),
      'Cmd-/': () => this.codeMirror.execCommand('toggleComment'),
    };

    this.codeMirror.addKeyMap(keyMap)
  }

  // Prevent the page from scrolling up
  // and scroll it manually to the bottom
  // on form submit.
  freezeScrollOnNextRender(event) {
    document.addEventListener('turbo:render', scrollToBottom);
  }

  // Disable cell until execution completes.
  // Prevents duplicate submits.
  play(event) {
    this.runningTarget.classList.remove('d-none')

    if (this.hasExecutionTimeTarget) {
      this.executionTimeTarget.classList.add('d-none')
    }

    if (this.codeMirror) {
      const disableKeyMap = {
        'Ctrl-Enter': () => null,
        'Cmd-Enter': () => null,
        'Ctrl-/': () => null,
        'Cmd-/': () => null,
      };

      this.codeMirror.setOption('readOnly', true)
      this.codeMirror.addKeyMap(disableKeyMap)
    }
  }

  cancelEdit(event) {
    event.preventDefault()
    this.cancelEditTarget.requestSubmit()
  }

  setSyntax(syntax) {
    this.codeMirror.setOption('mode', syntax)

    let cellType = 3
    if (syntax === 'gfm') {
      cellType = 1
    }

    this.cellTypeTarget.value = cellType
  }
}

const scrollToBottom = () => {
  window.Turbo.navigator.currentVisit.scrolled = true;
  window.scrollTo(0, document.body.scrollHeight)
  document.removeEventListener('turbo:render', scrollToBottom);
};
