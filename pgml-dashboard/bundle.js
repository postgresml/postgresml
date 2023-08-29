import { Controller, Application } from '@hotwired/stimulus';
import { renderDistribution, renderCorrelation, renderOutliers } from '@postgresml/main';

class ConfirmModalController extends Controller {
  connect() {
    
  }
}

class ModalController extends Controller {
  static targets = [
    'modal',
  ];

  connect() {
    this.modal = new bootstrap.Modal(this.modalTarget);
  }

  show() {
    this.modal.show();
  }

  hide() {
    this.modal.hide();
  }
}

class AutoreloadFrame extends Controller {
    static targets = [
        'frame',
    ];

    connect() {
        let interval = 5000; // 5 seconds

        if (this.hasFrameTarget) {
            this.frameTarget.querySelector('turbo-frame');

            if (this.frameTarget.dataset.interval) {
                let value = parseInt(this.frameTarget.dataset.interval);
                if (!isNaN(value)) {
                    interval = value;
                }
            }
        }

        if (this.hasFrameTarget) {
            const frame = this.frameTarget.querySelector('turbo-frame');

            if (frame) {
                this.interval = setInterval(() => {
                    const frame = this.frameTarget.querySelector('turbo-frame');
                    const src = `${frame.src}`;
                    frame.src = null;
                    frame.src = src;
                }, interval);
            }
        }
    }

    disconnect() {
        clearTimeout(this.interval);
    }
}

class BtnSecondary extends Controller {
    static targets = [
        'btnSecondary',
    ]

    connect() {
        this.respondToVisibility();
    }

    // Hook for when the secondary btn is in viewport
    respondToVisibility() {  
        let options = {
            root: null, 
            rootMargin: "0px"
        };

        var observer = new IntersectionObserver((entries) => {
            entries.forEach((entry) => {
                if (entry.isIntersecting) {
                    this.attachCanvas();
                }
            });
        }, options);

        observer.observe(this.btnSecondaryTarget);
    }
    
    attachCanvas() {
        let btn = this.btnSecondaryTarget;
        let canvasElements = btn.getElementsByTagName("canvas");

        if (canvasElements.length) {
            var canvas = canvasElements[0];
        } else {
            var canvas = document.createElement("canvas");
            canvas.className = "secondary-btn-canvas";
        }
        
        btn.appendChild(canvas);
        this.drawBorder(btn, canvas);
    }

    drawBorder(btn, canvas) {
        let btnMarginX = 22;
        let btnMarginY = 12;
        let borderRadius = 8;
        let width = btn.offsetWidth;
        let height = btn.offsetHeight;

    
        canvas.width = width;
        canvas.height = height;
        canvas.style.margin = `-${height - btnMarginY}px -${btnMarginX}px`;
        if( !width ) {
            return
        }
        
        // Draw border compensating for border thickenss
        var ctx = canvas.getContext("2d");
        ctx.moveTo(borderRadius, 1);
        ctx.lineTo(width-borderRadius-1, 1);
        ctx.arcTo(width-1, 1, width-1, borderRadius-1, borderRadius-1);
        ctx.arcTo(width-1, height-1, width-borderRadius-1, height-1, borderRadius-1);
        ctx.lineTo(borderRadius-1, height-1);
        ctx.arcTo(1, height-1, 1, borderRadius-1, borderRadius-1);
        ctx.arcTo(1, 1, borderRadius-1, 1, borderRadius-1);
    
        var gradient = ctx.createLinearGradient(0, canvas.height, canvas.width, 0);
        gradient.addColorStop(0, "rgb(217, 64, 255)");
        gradient.addColorStop(0.24242424242424243, "rgb(143, 2, 254)");
        gradient.addColorStop(0.5606060606060606, "rgb(81, 98, 255)");
        gradient.addColorStop(1, "rgb(0, 209, 255)");
      
        // Fill with gradient
        ctx.strokeStyle = gradient;
        ctx.lineWidth = 2;
        ctx.stroke();
    }
}

// Gym controller.


class ClickReplace extends Controller {
    static targets = [
        'frame',
    ];

    click(event) {
        let href = event.currentTarget.dataset.href;
        this.frameTarget.src = href;
    }
}

class Console extends Controller {
  static targets = [
    "code",
    "result",
    "run",
    "history",
    "resultSection",
    "historySection",
  ]

  connect() {
    this.myCodeMirror = CodeMirror.fromTextArea(document.getElementById("codemirror-console"), {
      value: "SELECT 1\n",
      mode:  "sql",
      lineNumbers: true,
    });

    this.history = [];
  }

  runQuery(event) {
    event.preventDefault();

    const query = event.currentTarget.querySelector("code").innerHTML;

    this.myCodeMirror.setValue(query);
    this.run(event, query);
  }

  addQueryToHistory(query) {
    this.history.push(query);

    if (this.history.length > 10) {
      this.history.shift();
    }

    let innerHTML = "";

    // Templates? Please. React? Nah.
    for (let query of this.history.reverse()) {
      innerHTML += `
        <li >
          <a href="#query-results" data-action="click->console#runQuery">
            <span><code>${query}</code></span>
          </a>
        </li>
      `;
    }

    this.historyTarget.innerHTML = innerHTML;
    this.historySectionTarget.classList.remove("hidden");
  }


  run(event, query) {
    this.runTarget.disabled = true;
    this.resultSectionTarget.classList.remove("hidden");
    this.resultTarget.innerHTML = "Running...";

    if (!query) {
      query = this.myCodeMirror.getValue();
      this.addQueryToHistory(query);
    }

    myFetch(`/console/run/`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      redirect: "follow",
      body: JSON.stringify({
        "query": query,
      }),
    })
    .then(res => res.text())
    .then(html => {
      this.resultTarget.innerHTML = html;
      this.runTarget.disabled = false;
    });
  }
}

function createToast(message) {
    const toastElement = document.createElement('div');
    toastElement.classList.add('toast', 'hide');
    toastElement.setAttribute('role', 'alert');
    toastElement.setAttribute('aria-live', 'assertive');
    toastElement.setAttribute('aria-atomic', 'true');

    const toastBodyElement = document.createElement('div');
    toastBodyElement.classList.add('toast-body');
    toastBodyElement.innerHTML = message;

    toastElement.appendChild(toastBodyElement);

    const container = document.getElementById('toast-container');
    container.appendChild(toastElement);

    // remove from DOM when no longer needed
    toastElement.addEventListener('hidden.bs.toast', (e) => e.target.remove());

    return toastElement
}


function showToast(toastElement) {
    const config = {
        'autohide': true,
        'delay': 2000,
    };
    const toastBootstrap = bootstrap.Toast.getOrCreateInstance(toastElement, config);
    toastBootstrap.show();
}

class Copy extends Controller {
    codeCopy() {
        let text = [...this.element.querySelectorAll('span.code-content')]
            .map((copied) => copied.innerText)
            .join('\n');

        if (text.length === 0) {
            text = this.element.innerText.replace('content_copy', '');
        }

        text = text.trim();

        navigator.clipboard.writeText(text);

        const toastElement = createToast('Copied to clipboard');
        showToast(toastElement);
    }

}

class DocsToc extends Controller {
  connect() {
    this.scrollSpyAppend();
  }

  scrollSpyAppend() {
    new bootstrap.ScrollSpy(document.body, {
      target: '#toc-nav',
      smoothScroll: true,
      rootMargin: '-10% 0% -50% 0%',
      threshold: [1],
    });
  }
}

class EnableTooltip extends Controller {
    connect() {
        const tooltipTriggerList = this.element.querySelectorAll('[data-bs-toggle="tooltip"]');
        [...tooltipTriggerList].map(tooltipTriggerEl => new bootstrap.Tooltip(tooltipTriggerEl));
    }
}

// extends bootstraps collapse component by adding collapse state class to any 
// html element you like.  This is useful for adding style changes to elements 
// that do not need to collapse, when a collapse state change occures. 

class ExtendBsCollapse extends Controller {

    static targets = [
        'stateReference'
    ]

    static values = {
        affected: String
    }

    connect() {
        this.navStates = ['collapsing', 'collapsed', 'expanding', 'expanded'];
        this.events = ['hide.bs.collapse', 'hidden.bs.collapse', 'show.bs.collapse', 'shown.bs.collapse'];

        this.events.forEach(event => {
            this.stateReferenceTarget.addEventListener(event, () => {
                this.getAllAffected().forEach(item => this.toggle(item));
            });
        });
    }

    getAllAffected() {
        return this.element.querySelectorAll(this.affectedValue)
    }

    toggle(item) {
        for (const [index, state] of this.navStates.entries()) {
            if( item.classList.contains(state)) {
                this.changeClass(this.navStates[(index+1)%4], item);
                return
            }
        }
    }

    changeClass(eClass, item) {
        this.navStates.map(c => item.classList.remove(c));
        item.classList.add(eClass);
    }

}

class NewProject extends Controller {
    static targets = [
      "step",
      "progressBar",
      "progressBarAmount",
      "sample",
      "tableStatus",
      "dataSourceNext",
      "projectStatus",
      "task",
      "taskNameNext",
      "projectNameNext",
      "trainingLabel",
      "analysisNext",
      "algorithmListClassification",
      "algorithmListRegression",
      "analysisResult",
      "projectError",
    ]

    initialize() {
        this.index = 0;
        this.targetNames = new Set();
        this.algorithmNames = new Set();

        this.checkDataSourceDebounced = _.debounce(this.checkDataSource, 250);
        this.checkProjectNameDebounced = _.debounce(this.checkProjectName, 250);
    }

    renderSteps() {
        this.stepTargets.forEach((element, index) => {
            if (index !== this.index)
                element.classList.add("hidden");
            else
                element.classList.remove("hidden");
        });
    }

    renderProgressBar() {
      // Let's get stuck on 97 just like Windows Update... ;)
      if (this.progressBarInterval && this.progressBarProgress >= 95)
        clearInterval(this.progressBarInterval);

      this.progressBarProgress += 2;
      const progress = Math.min(100, this.progressBarProgress);

      this.progressBarTarget.style = `width: ${progress > 0 ? progress : "auto"}%;`;
      this.progressBarAmountTarget.innerHTML = `${progress}%`;
    }

    checkDataSource(event) {
        let tableName = event.target.value;

        myFetch(`/api/tables/?table_name=${tableName}`)
        .then(res => {
            if (res.ok) {
                this.tableName = tableName;
                this.renderSample();
                this.renderTarget();
            }
            else {
                this.tableName = null;
                this.sampleTarget.innerHTML = "";
                this.trainingLabelTarget.innerHTML = "";
              }
            this.renderTableStatus();
        })
        .catch(err => {
            this.tableName = null;
            this.renderTableStatus();
        });
    }

    checkProjectName(event) {
      let projectName = event.target.value;

      myFetch(`/api/projects/?name=${projectName}`)
      .then(res => res.json())
      .then(json => {
        if (json.results.length > 0) {
          this.projectName = null;
        } else {
          this.projectName = projectName;
        }

        this.renderProjectStatus();
      });
    }

    selectTask(event) {
      event.preventDefault();

      this.taskName = event.currentTarget.dataset.task;

      if (this.taskName  === "regression") {
        this.algorithmListClassificationTarget.classList.add("hidden");
        this.algorithmListRegressionTarget.classList.remove("hidden");
      } else if (this.taskName  == "classification") {
        this.algorithmListClassificationTarget.classList.remove("hidden");
        this.algorithmListRegressionTarget.classList.add("hidden");
      }

      this.taskTargets.forEach(task => {
        task.classList.remove("selected");
      });

      event.currentTarget.classList.add("selected");
      this.taskNameNextTarget.disabled = false;
    }

    selectAlgorithm(event) {
      event.preventDefault();

      let algorithmName = event.currentTarget.dataset.algorithm;

      if (event.currentTarget.classList.contains("selected")) {
        event.currentTarget.classList.remove("selected");
        this.algorithmNames.delete(algorithmName);
      } else {
        event.currentTarget.classList.add("selected");
        this.algorithmNames.add(algorithmName);
      }

    }

    renderTableStatus() {
        if (this.tableName) {
            this.tableStatusTarget.innerHTML = "done";
            this.tableStatusTarget.classList.add("ok");
            this.tableStatusTarget.classList.remove("error");
            this.dataSourceNextTarget.disabled = false;
        } else {
            this.tableStatusTarget.innerHTML = "close";
            this.tableStatusTarget.classList.add("error");
            this.tableStatusTarget.classList.remove("ok");
            this.dataSourceNextTarget.disabled = true;
        }
        
    }

    renderProjectStatus() {
      if (this.projectName) {
            this.projectStatusTarget.innerHTML = "done";
            this.projectStatusTarget.classList.add("ok");
            this.projectStatusTarget.classList.remove("error");
            this.projectNameNextTarget.disabled = false;
        } else {
            this.projectStatusTarget.innerHTML = "close";
            this.projectStatusTarget.classList.add("error");
            this.projectStatusTarget.classList.remove("ok");
            this.projectNameNextTarget.disabled = true;
        }
    }

    renderSample() {
        myFetch(`/api/tables/sample/?table_name=${this.tableName}`)
        .then(res => res.text())
        .then(html => this.sampleTarget.innerHTML = html);
    }

    renderTarget() {
      myFetch(`/api/tables/columns/?table_name=${this.tableName}`)
      .then(res => res.text())
      .then(html => this.trainingLabelTarget.innerHTML = html);
    }

    renderAnalysisResult() {
      const snapshotData = this.projectData.models[0].snapshot;

      console.log("Fetching analysis");
      myFetch(`/html/snapshots/analysis/?snapshot_id=${snapshotData.id}`)
      .then(res => res.text())
      .then(html => this.analysisResultTarget.innerHTML = html)
      .then(() => {
        // Render charts
        for (let name in snapshotData.columns) {
          const sample = JSON.parse(document.getElementById(name).textContent);
          renderDistribution(name, sample, snapshotData.analysis[`${name}_dip`]);

          for (let target of snapshotData.y_column_name) {
            if (target === name)
              continue

            const targetSample = JSON.parse(document.getElementById(target).textContent);
            renderCorrelation(name, target, sample, targetSample);
          }
        }

        for (let target of snapshotData.y_column_name) {
          const targetSample = JSON.parse(document.getElementById(target).textContent);
          renderOutliers(target, targetSample, snapshotData.analysis[`${target}_stddev`]);
        }

        this.progressBarProgress = 100;
        this.renderProgressBar();

        setTimeout(this.nextStep.bind(this), 1000);
      });
    }

    selectTarget(event) {
      event.preventDefault();
      let targetName = event.currentTarget.dataset.columnName;

      if (event.currentTarget.classList.contains("selected")) {
        this.targetNames.delete(targetName);
        event.currentTarget.classList.remove("selected");
      } else {
        this.targetNames.add(targetName);
        event.currentTarget.classList.add("selected");
      }

      if (this.targetNames.size > 0)
        this.analysisNextTarget.disabled = false;
      else
        this.analysisNextTarget.disabled = true;
    }

    createSnapshot(event) {
      event.preventDefault();

      // Train a linear algorithm by default
      this.algorithmNames.add("linear");

      this.nextStep();

      // Start the progress bar :)
      this.progressBarProgress = 2;
      this.progressBarInterval = setInterval(this.renderProgressBar.bind(this), 850);

      this.createProject(event, false, () => {
        this.index += 1; // Skip error page
        this.renderAnalysisResult();
        this.algorithmNames.delete("linear");
      });
    }

    createProject(event, redirect = true, callback = null) {
      event.preventDefault();

      const request = {
        "project_name": this.projectName,
        "task": this.taskName,
        "algorithms": Array.from(this.algorithmNames),
        "relation_name": this.tableName,
        "y_column_name": Array.from(this.targetNames),
      };

      if (redirect)
        this.createLoader();

      myFetch(`/api/projects/train/`, {
        method: "POST",
        cache: "no-cache",
        headers: {
          "Content-Type": "application/json",
        },
        redirect: "follow",
        body: JSON.stringify(request),
      })
      .then(res => {
        if (res.ok) {
          return res.json()
        } else {
          const json = res.json().then((json) => {
            clearInterval(this.progressBarInterval);
            this.projectErrorTarget.innerHTML = json.error;
            this.nextStep();
          });
          throw Error(`Failed to train project: ${json.error}`)
        }
      })
      .then(json => {
        this.projectData = json;

        if (redirect)
          window.location.assign(`/${window.urlPrefix}/projects/${json.id}`);

        if (callback)
          callback();
      });
    }

    createLoader() {
      let element = document.createElement("div");
      element.innerHTML = `
        <div id="loader">
          <div class="loader"></div>
        </div>
      `;
      document.body.appendChild(element);
    }

    nextStep() {
        this.index += 1;
        this.renderSteps();
    }

    previousStep() {
        this.index -= 1;
        this.renderSteps();
    }

    restart() {
      this.index = 0;
      this.renderSteps();
    }
}

class NotebookCell extends Controller {
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
      this.initCodeMirrorOnTarget(this.editorTarget);
    }

    if (this.cellTarget.dataset.cellState === 'new') {
      this.cellTarget.scrollIntoView();
    }

    this.cellTarget.addEventListener('mouseover', this.showDragAndDrop.bind(this));
    this.cellTarget.addEventListener('mouseout', this.hideDragAndDrop.bind(this));
  }

  showDragAndDrop(event) {
    this.dragAndDropTarget.classList.remove('d-none');
  }

  hideDragAndDrop(event) {
    this.dragAndDropTarget.classList.add('d-none');
  }

  // Enable CodeMirror on target.
  initCodeMirrorOnTarget(target) {
    let mode = 'sql';

    if (target.dataset.type === 'markdown') {
      mode = 'gfm';
    }

    this.codeMirror = CodeMirror.fromTextArea(target, {
      lineWrapping: true,
      matchBrackets: true,
      mode,
      scrollbarStyle: 'null',
      lineNumbers: mode === 'sql',
    });

    this.codeMirror.setSize('100%', 'auto');

    const keyMap = {
      'Ctrl-Enter': () => this.formTarget.requestSubmit(),
      'Cmd-Enter': () => this.formTarget.requestSubmit(),
      'Ctrl-/': () => this.codeMirror.execCommand('toggleComment'),
      'Cmd-/': () => this.codeMirror.execCommand('toggleComment'),
    };

    this.codeMirror.addKeyMap(keyMap);
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
    this.runningTarget.classList.remove('d-none');

    if (this.hasExecutionTimeTarget) {
      this.executionTimeTarget.classList.add('d-none');
    }

    if (this.codeMirror) {
      const disableKeyMap = {
        'Ctrl-Enter': () => null,
        'Cmd-Enter': () => null,
        'Ctrl-/': () => null,
        'Cmd-/': () => null,
      };

      this.codeMirror.setOption('readOnly', true);
      this.codeMirror.addKeyMap(disableKeyMap);
    }
  }

  cancelEdit(event) {
    event.preventDefault();
    this.cancelEditTarget.requestSubmit();
  }

  setSyntax(syntax) {
    this.codeMirror.setOption('mode', syntax);

    let cellType = 3;
    if (syntax === 'gfm') {
      cellType = 1;
    }

    this.cellTypeTarget.value = cellType;
  }
}

const scrollToBottom = () => {
  window.Turbo.navigator.currentVisit.scrolled = true;
  window.scrollTo(0, document.body.scrollHeight);
  document.removeEventListener('turbo:render', scrollToBottom);
};

class Notebook extends Controller {
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

  cellCheckIntervalMillis = 500

  connect() {
    document.addEventListener('keyup', this.executeSelectedCell.bind(this));
    const rect = this.scrollerTarget.getBoundingClientRect();
    const innerHeight = window.innerHeight;

    this.scrollerTarget.style.maxHeight = `${innerHeight - rect.top - 10}px`;
    // this.confirmDeleteModal = new bootstrap.Modal(this.deleteModalTarget)

    this.sortable = Sortable.create(this.scrollerTarget, {
      onUpdate: this.updateCellOrder.bind(this),
      onStart: this.makeCellsReadOnly.bind(this),
      onEnd: this.makeCellsEditable.bind(this),
    });
  }

  disconnect() {
    document.removeEventListener('keyup', this.executeSelectedCell.bind(this));
  }

  makeCellsReadOnly(event) {
    this.codeMirrorReadOnly(true);
  }

  makeCellsEditable(event) {
    this.codeMirrorReadOnly(false);
  }

  codeMirrorReadOnly(readOnly) {
    const cells = document.querySelectorAll(`div[data-cell-id]`);

    cells.forEach(cell => {
      const controller = this.application.getControllerForElementAndIdentifier(cell, 'notebook-cell');
      if (controller.codeMirror) {
        controller.codeMirror.setOption('readOnly', readOnly);
      }
    });
  }

  updateCellOrder(event) {
    const cells = [...this.scrollerTarget.querySelectorAll('turbo-frame')];
    const notebookId = this.scrollerTarget.dataset.notebookId;
    const ids = cells.map(cell => parseInt(cell.dataset.cellId));

    fetch(`/dashboard/notebooks/${notebookId}/reorder`, {
      method: 'POST',
      body: JSON.stringify({
        cells: ids,
      }),
      headers: {
        'Content-Type': 'application/json',
      }
    });

    this.scrollerTarget.querySelectorAll('div[data-cell-number]').forEach((cell, index) => {
      cell.dataset.cellNumber = index + 1;
      cell.innerHTML = index + 1;
    });
  }

  playAll(event) {
    event.currentTarget.disabled = true;
    const frames = this.scrollerTarget.querySelectorAll('turbo-frame[data-cell-type="3"]');
    this.playCells([...frames]);
  }

  playCells(frames) {
    const frame = frames.shift();
    const form = document.querySelector(`form[data-cell-play-id="${frame.dataset.cellId}"]`);
    const cellType = form.querySelector('input[name="cell_type"]').value;
    const contents = form.querySelector('textarea[name="contents"]').value;
    const body = `cell_type=${cellType}&contents=${encodeURIComponent(contents)}`;

    fetch(form.action, {
      method: 'POST',
      body,
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8',
      },
    }).then(response => {
      // Reload turbo frame
      frame.querySelector('a[data-notebook-target="loadCell"]').click();

      if (response.status > 300) {
        throw new Error(response.statusText)
      }

      if (frames.length > 0) {
        setTimeout(() => this.playCells(frames), 250);
      } else {
        this.playAllButtonTarget.disabled = false;
      }
    });
  }

  // Check that the cell finished running.
  // We poll the DOM every 500ms. Not a very clever solution, but I think it'll work.
  checkCellState() {
    const cell = document.querySelector(`div[data-cell-id="${this.activeCellId}"]`);

    if (cell.dataset.cellState === 'rendered') {
      this.playButtonTextTarget.innerHTML = 'Run';
      clearInterval(this.cellCheckInterval);
      this.enableCellButtons();
      this.stopButtonTarget.disabled = true;
    }
  }

  playCell(event) {
    // Start execution.
    const cell = document.querySelector(`div[data-cell-id="${this.activeCellId}"]`);

    const form = cell.querySelector(`form[data-cell-play-id="${this.activeCellId}"]`);
    form.requestSubmit();
    
    if (cell.dataset.cellType === '3') {
      this.playButtonTextTarget.innerHTML = 'Running';
      this.disableCellButtons();
      
      cell.dataset.cellState = 'running';

      // Check on the result of the cell every 500ms.
      this.cellCheckInterval = setInterval(this.checkCellState.bind(this), this.cellCheckIntervalMillis);

      // Enable the stop button if we're running code.
      this.stopButtonTarget.disabled = false;
    }
  }

  playStop() {
    this.stopButtonTarget.disabled = true;
    this.disableCellButtons();

    const form = document.querySelector(`form[data-cell-stop-id="${this.activeCellId}"]`);
    form.requestSubmit();

    // The query will be terminated immediately, unless there is a real problem.
    this.enableCellButtons();
  }

  enableCellButtons() {
    this.cellButtonTargets.forEach(target => target.disabled = false);
  }

  disableCellButtons() {
    this.cellButtonTargets.forEach(target => target.disabled = true);
  }

  selectCell(event) {
    if (event.currentTarget.classList.contains('active')) {
      return
    }

    this.enableCellButtons();
    this.activeCellId = event.currentTarget.dataset.cellId;

    this.cellTargets.forEach(target => {
      if (target.classList.contains('active')) {
        // Reload the cell from the backend, i.e. cancel the edit.
        target.querySelector('a[data-notebook-target="loadCell"]').click();
      }
    });

    if (!event.currentTarget.classList.contains('active')) {
      event.currentTarget.classList.add('active');
    }

    let cellType = 'SQL';
    if (event.currentTarget.dataset.cellType === '1') {
      cellType = 'Markdown';
    }

    this.syntaxNameTarget.innerHTML = cellType;
  }

  executeSelectedCell(event) {
    if (!this.activeCellId) {
      return
    }

    if (event.shiftKey) {
      if (event.key === 'Enter' && event.keyCode === 13) {
        this.playCell();
      }
    }
  }

  deleteCellConfirm() {
    this.modalOutlet.show();
  }

  deleteCell() {
    const form = document.querySelector(`form[data-cell-delete-id="${this.activeCellId}"]`);
    form.requestSubmit();
  }

  newCell() {
    this.newCellTarget.requestSubmit();
  }

  changeSyntax(event) {
    event.preventDefault();
    const syntax = event.currentTarget.dataset.syntax;

    const cell = document.querySelector(`div[data-cell-id="${this.activeCellId}"]`);
    const controller = this.application.getControllerForElementAndIdentifier(cell, 'notebook-cell');
    controller.setSyntax(event.currentTarget.dataset.syntax);

    if (syntax === 'gfm') {
      this.syntaxNameTarget.innerHTML = 'Markdown';
    } else {
      this.syntaxNameTarget.innerHTML = 'SQL';
    }
  }
}

class QuickPrediction extends Controller {
  static targets = [
    "feature",
    "step",
    "prediction",
  ]

  initialize() {
    this.index = 0;
  }

  nextStep() {
    this.index += 1;
    this.renderSteps();
  }

  prevStep() {
    this.index -= 1;
    this.renderSteps();
  }

  renderSteps() {
    this.stepTargets.forEach((element, index) => {
      if (this.index !== index) {
        element.classList.add("hidden");
      } else {
        element.classList.remove("hidden");
      }
    });
  }

  predict(event) {
    const inputs = [];

    this.featureTargets.forEach(target => {
      target.getAttribute("name");
      const value = target.value;

      inputs.push(Number(value));
    });

    const modelId = event.currentTarget.dataset.modelId;

    myFetch(`/api/models/${modelId}/predict/`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(inputs),
    })
    .then(res => res.json())
    .then(json => {
      this.predictionTargets.forEach((element, index) => {
        element.innerHTML = json.predictions[index];
      });
      this.nextStep();
    });
  }
}

class Search extends Controller {
    static targets = [
        'searchTrigger',
    ]

    connect() {
        this.target = document.getElementById("search");
        this.searchInput = document.getElementById("search-input");
        this.searchFrame = document.getElementById("search-results");

        this.target.addEventListener('shown.bs.modal', this.focusSearchInput);
        this.target.addEventListener('hidden.bs.modal', this.updateSearch);
        this.searchInput.addEventListener('input', (e) => this.search(e));
    }

    search(e) {
        const query = e.currentTarget.value;
        this.searchFrame.src = `/docs/search?query=${query}`;
    }

    focusSearchInput = (e) => {
        this.searchInput.focus();
        this.searchTriggerTarget.blur();
    }

    updateSearch = () => {
      this.searchTriggerTarget.value = this.searchInput.value;
    }

    openSearch = (e) => {
      new bootstrap.Modal(this.target).show();
      this.searchInput.value = e.currentTarget.value;
    }

    disconnect() {
        this.searchTriggerTarget.removeEventListener('shown.bs.modal', this.focusSearchInput);
        this.searchTriggerTarget.removeEventListener('hidden.bs.modal', this.updateSearch);
    }
}

// from https://github.com/afcapel/stimulus-autocomplete/blob/main/src/autocomplete.js

const optionSelector = "[role='option']:not([aria-disabled])";
const activeSelector = "[aria-selected='true']";

class Autocomplete extends Controller {
  static targets = ["input", "hidden", "results"]
  static classes = ["selected"]
  static values = {
    ready: Boolean,
    submitOnEnter: Boolean,
    url: String,
    minLength: Number,
    delay: { type: Number, default: 300 },
  }
  static uniqOptionId = 0

  connect() {
    this.close();

    if(!this.inputTarget.hasAttribute("autocomplete")) this.inputTarget.setAttribute("autocomplete", "off");
    this.inputTarget.setAttribute("spellcheck", "false");

    this.mouseDown = false;

    this.onInputChange = debounce(this.onInputChange, this.delayValue);

    this.inputTarget.addEventListener("keydown", this.onKeydown);
    this.inputTarget.addEventListener("blur", this.onInputBlur);
    this.inputTarget.addEventListener("input", this.onInputChange);
    this.resultsTarget.addEventListener("mousedown", this.onResultsMouseDown);
    this.resultsTarget.addEventListener("click", this.onResultsClick);

    if (this.inputTarget.hasAttribute("autofocus")) {
      this.inputTarget.focus();
    }

    this.readyValue = true;
  }

  disconnect() {
    if (this.hasInputTarget) {
      this.inputTarget.removeEventListener("keydown", this.onKeydown);
      this.inputTarget.removeEventListener("blur", this.onInputBlur);
      this.inputTarget.removeEventListener("input", this.onInputChange);
    }

    if (this.hasResultsTarget) {
      this.resultsTarget.removeEventListener("mousedown", this.onResultsMouseDown);
      this.resultsTarget.removeEventListener("click", this.onResultsClick);
    }
  }

  sibling(next) {
    const options = this.options;
    const selected = this.selectedOption;
    const index = options.indexOf(selected);
    const sibling = next ? options[index + 1] : options[index - 1];
    const def = next ? options[0] : options[options.length - 1];
    return sibling || def
  }

  select(target) {
    const previouslySelected = this.selectedOption;
    if (previouslySelected) {
      previouslySelected.removeAttribute("aria-selected");
      previouslySelected.classList.remove(...this.selectedClassesOrDefault);
    }

    target.setAttribute("aria-selected", "true");
    target.classList.add(...this.selectedClassesOrDefault);
    this.inputTarget.setAttribute("aria-activedescendant", target.id);
    target.scrollIntoView({ behavior: "smooth", block: "nearest" });
  }

  onKeydown = (event) => {
    const handler = this[`on${event.key}Keydown`];
    if (handler) handler(event);
  }

  onEscapeKeydown = (event) => {
    if (!this.resultsShown) return

    this.hideAndRemoveOptions();
    event.stopPropagation();
    event.preventDefault();
  }

  onArrowDownKeydown = (event) => {
    const item = this.sibling(true);
    if (item) this.select(item);
    event.preventDefault();
  }

  onArrowUpKeydown = (event) => {
    const item = this.sibling(false);
    if (item) this.select(item);
    event.preventDefault();
  }

  onTabKeydown = (event) => {
    const selected = this.selectedOption;
    if (selected) this.commit(selected);
  }

  onEnterKeydown = (event) => {
    const selected = this.selectedOption;
    if (selected && this.resultsShown) {
      this.commit(selected);
      if (!this.hasSubmitOnEnterValue) {
        event.preventDefault();
      }
    }
  }

  onInputBlur = () => {
    if (this.mouseDown) return
    this.close();
  }

  commit(selected) {
    if (selected.getAttribute("aria-disabled") === "true") return

    if (selected instanceof HTMLAnchorElement) {
      selected.click();
      this.close();
      return
    }

    const textValue = selected.getAttribute("data-autocomplete-label") || selected.textContent.trim();
    const value = selected.getAttribute("data-autocomplete-value") || textValue;
    this.inputTarget.value = textValue;

    if (this.hasHiddenTarget) {
      this.hiddenTarget.value = value;
      this.hiddenTarget.dispatchEvent(new Event("input"));
      this.hiddenTarget.dispatchEvent(new Event("change"));
    } else {
      this.inputTarget.value = value;
    }

    this.inputTarget.focus();
    this.hideAndRemoveOptions();

    this.element.dispatchEvent(
      new CustomEvent("autocomplete.change", {
        bubbles: true,
        detail: { value: value, textValue: textValue, selected: selected }
      })
    );
  }

  clear() {
    this.inputTarget.value = "";
    if (this.hasHiddenTarget) this.hiddenTarget.value = "";
  }

  onResultsClick = (event) => {
    if (!(event.target instanceof Element)) return
    const selected = event.target.closest(optionSelector);
    if (selected) this.commit(selected);
  }

  onResultsMouseDown = () => {
    this.mouseDown = true;
    this.resultsTarget.addEventListener("mouseup", () => {
      this.mouseDown = false;
    }, { once: true });
  }

  onInputChange = () => {
    this.element.removeAttribute("value");
    if (this.hasHiddenTarget) this.hiddenTarget.value = "";

    const query = this.inputTarget.value.trim();
    if (query && query.length >= this.minLengthValue) {
      this.fetchResults(query);
    } else {
      this.hideAndRemoveOptions();
    }
  }

  identifyOptions() {
    const prefix = this.resultsTarget.id || "stimulus-autocomplete";
    const optionsWithoutId = this.resultsTarget.querySelectorAll(`${optionSelector}:not([id])`);
    optionsWithoutId.forEach(el => el.id = `${prefix}-option-${Autocomplete.uniqOptionId++}`);
  }

  hideAndRemoveOptions() {
    this.close();
    this.resultsTarget.innerHTML = null;
  }

  fetchResults = async (query) => {
    if (!this.hasUrlValue) return

    const url = this.buildURL(query);
    try {
      this.element.dispatchEvent(new CustomEvent("loadstart"));
      const html = await this.doFetch(url);
      this.replaceResults(html);
      this.element.dispatchEvent(new CustomEvent("load"));
      this.element.dispatchEvent(new CustomEvent("loadend"));
    } catch(error) {
      this.element.dispatchEvent(new CustomEvent("error"));
      this.element.dispatchEvent(new CustomEvent("loadend"));
      throw error
    }
  }

  buildURL(query) {
    const url = new URL(this.urlValue, window.location.href);
    const params = new URLSearchParams(url.search.slice(1));
    params.append("q", query);
    url.search = params.toString();

    return url.toString()
  }

  doFetch = async (url) => {
    const response = await myFetch(url, this.optionsForFetch());
    const html = await response.text();
    return html
  }

  replaceResults(html) {
    this.resultsTarget.innerHTML = html;
    this.identifyOptions();
    if (!!this.options) {
      this.open();
    } else {
      this.close();
    }
  }

  open() {
    if (this.resultsShown) return

    this.resultsShown = true;
    this.element.setAttribute("aria-expanded", "true");
    this.element.dispatchEvent(
      new CustomEvent("toggle", {
        detail: { action: "open", inputTarget: this.inputTarget, resultsTarget: this.resultsTarget }
      })
    );
  }

  close() {
    if (!this.resultsShown) return

    this.resultsShown = false;
    this.inputTarget.removeAttribute("aria-activedescendant");
    this.element.setAttribute("aria-expanded", "false");
    this.element.dispatchEvent(
      new CustomEvent("toggle", {
        detail: { action: "close", inputTarget: this.inputTarget, resultsTarget: this.resultsTarget }
      })
    );
  }

  get resultsShown() {
    return !this.resultsTarget.hidden
  }

  set resultsShown(value) {
    this.resultsTarget.hidden = !value;
  }

  get options() {
    return Array.from(this.resultsTarget.querySelectorAll(optionSelector))
  }

  get selectedOption() {
    return this.resultsTarget.querySelector(activeSelector)
  }

  get selectedClassesOrDefault() {
    return this.hasSelectedClass ? this.selectedClasses : ["active"]
  }

  optionsForFetch() {
    return { headers: { "X-Requested-With": "XMLHttpRequest" } } // override if you need
  }
}

const debounce = (fn, delay = 10) => {
  let timeoutId = null;

  return (...args) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(fn, delay);
  }
};

class Timeseries extends Controller {

    static values = {
        metricData: Object
    }

    connect() {
        // Plot on load and refresh button
        this.plot();

        // resize on navigation to metric tab
        const tabElement = document.querySelector('button[data-bs-target="#tab-Metrics"]');
        tabElement.addEventListener('shown.bs.tab', event => {
            this.plot();
        }, {once: true});
    }

    plot() {            
        const min = Math.min(...this.metricDataValue.values);
        const max = Math.max(...this.metricDataValue.values);
        const range = max-min;
        const color = "#ABACB0";
        const activeColor = "#F8FAFC";
        const lineColor = "#9185FF";
        const bgColor = "transparent";

        const trace = {
            x: this.metricDataValue.utc, 
            y: this.metricDataValue.values,
            fill: 'tonexty',
            mode: 'lines',
            line: {
              color: lineColor,
            },
        };

        const layout = {
            showlegend: false,
            plot_bgcolor: bgColor,
            paper_bgcolor: bgColor,
            height: document.body.offsetHeight*0.3,
            font: {
                color: color
            },
            margin: {b: 0, l: 0, r: 0, t: 40},
            yaxis: {
                range: [min-0.1*range, max+0.1*range],
                showgrid: false,
                automargin: true
            }, 
            xaxis: {
                showgrid: false,
                automargin: true
            },
            modebar: {
                activecolor: activeColor,
                bgcolor: bgColor,
                color: color,
                remove: ['autoscale', 'zoomin', 'zoomout']
            }
        };

        const config = {
            responsive: true,
            displaylogo: false
        };

        Plotly.newPlot(this.element.id, [trace], layout, config);
    }
}

class TopnavStyling extends Controller {
    initialize() {
        this.pinned_to_top = false;
    }
    
    connect() {
        this.act_when_scrolled();
        this.act_when_expanded();
    }

    act_when_scrolled() {
        // check scroll position in initial render
        if( window.scrollY > 48) {
            this.pinned_to_top = true; 
            this.element.classList.add("pinned");
        }

        addEventListener("scroll", (event) => {
            if (window.scrollY > 48 && !this.pinned_to_top) {
                this.pinned_to_top = true;
                this.element.classList.add("pinned");
            }
            
            if (window.scrollY < 48 && this.pinned_to_top) {
                this.pinned_to_top = false;
                this.element.classList.remove("pinned");
            }        });
    }

    // Applies a class when navbar is expanded, used in mobile view for adding background contrast.
    act_when_expanded() {
        addEventListener('show.bs.collapse', (e) => {
            if (e.target.id === 'navbarSupportedContent') {
                this.element.classList.add('navbar-expanded');
            }
        });
        addEventListener('hidden.bs.collapse', (e) => {
            if (e.target.id === 'navbarSupportedContent') {
                this.element.classList.remove('navbar-expanded');
            }
        });
    }
    
}

class TopnavWebApp extends Controller {

    connect() {
        let navbarMenues = document.querySelectorAll('.navbar-collapse');

        document.addEventListener('show.bs.collapse', e => {
            this.closeOtherMenues(navbarMenues, e.target);
        });

        document.addEventListener('hidden.bs.collapse', e => {
            this.closeSubmenus(e.target.querySelectorAll('.drawer-submenu'));
        });
    }

    closeOtherMenues(menus, current) {
        menus.forEach( menu => {
            const bsInstance = bootstrap.Collapse.getInstance(menu);
            if ( bsInstance && menu != current && menu != current.parentElement ) {
                bsInstance.hide();
            }
        });
    }

    closeSubmenus(submenues) {
        submenues.forEach(submenu => {
            const bsInstance = bootstrap.Collapse.getInstance(submenu);
            if ( bsInstance ) {
                bsInstance.hide();
            }
        });
    }
}

class XScrollerDrag extends Controller {
    isDown = false;
    startX;
    scrollLeft;

    static targets = [
        "slider"
    ]
    
    // TODO: Fix firefox highlight on grab.
    grab(e) {
        this.isDown = true;
        this.startX = e.pageX - this.sliderTarget.offsetLeft;
        this.scrollLeft = this.sliderTarget.scrollLeft;
    }

    leave() {
        this.isDown = false;
    }

    release() {
        this.isDown = false;
    }

    move(e) {
        if(!this.isDown) return;
        e.preventDefault();
        const x = e.pageX - this.sliderTarget.offsetLeft;
        const difference = (x - this.startX);
        this.sliderTarget.scrollLeft = this.scrollLeft - difference;
    }

}

const application = Application.start();
application.register('confirm-modal', ConfirmModalController);
application.register('modal', ModalController);
application.register('autoreload-frame', AutoreloadFrame);
application.register('btn-secondary', BtnSecondary);
application.register('click-replace', ClickReplace);
application.register('console', Console);
application.register('copy', Copy);
application.register('docs-toc', DocsToc);
application.register('enable-tooltip', EnableTooltip);
application.register('extend-bs-collapse', ExtendBsCollapse);
application.register('new-project', NewProject);
application.register('notebook-cell', NotebookCell);
application.register('notebook', Notebook);
application.register('quick-prediction', QuickPrediction);
application.register('search', Search);
application.register('stimulus-autocomplete', Autocomplete);
application.register('timeseries', Timeseries);
application.register('topnav-styling', TopnavStyling);
application.register('topnav-web-app', TopnavWebApp);
application.register('x-scroller-drag', XScrollerDrag);
