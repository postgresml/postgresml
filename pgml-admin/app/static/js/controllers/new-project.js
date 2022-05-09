import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
    static targets = ["step", "progressBar", "progressBarAmount", "sample", "tableStatus", "trainingLabel"];

    initialize() {
        this.index = 0
        this.targetNames = new Set()
    }

    renderSteps() {
        this.stepTargets.forEach((element, index) => {
            if (index !== this.index)
                element.classList.add("hidden")
            else
                element.classList.remove("hidden")
        })
    }

    renderProgressBar() {
        let progress = Math.ceil(this.index / this.stepTargets.length * 100)

        this.progressBarTarget.style = `width: ${progress > 0 ? progress : 'auto'}%;`
        this.progressBarAmountTarget.innerHTML = `${progress}%`
    }

    checkDataSource(event) {
        let tableName = event.target.value

        fetch(`/api/tables/?table_name=${tableName}`)
        .then(res => {
            if (res.ok) {
                this.tableName = tableName
                this.renderSample()
                this.renderTarget()
            }
            else
                this.tableName = null
            this.renderTableStatus()
        })
        .catch(err => {
            this.tableName = null
            this.renderTableStatus()
        })
    }

    checkProjectName(event) {
      let projectName = event.target.value
      this.projectName = projectName
    }

    renderTableStatus() {
        if (this.tableName) {
            this.tableStatusTarget.innerHTML = "done"
            this.tableStatusTarget.classList.add("ok")
            this.tableStatusTarget.classList.remove("error")
        } else {
            this.tableStatusTarget.innerHTML = "close"
            this.tableStatusTarget.classList.add("error")
            this.tableStatusTarget.classList.remove("ok")
        }
        
    }

    renderSample() {
        fetch(`/api/tables/sample/?table_name=${this.tableName}`)
        .then(res => res.text())
        .then(html => this.sampleTarget.innerHTML = html)
    }

    renderTarget() {
      fetch(`/api/tables/columns/?table_name=${this.tableName}`)
      .then(res => res.text())
      .then(html => this.trainingLabelTarget.innerHTML = html)
    }

    selectTarget(event) {
      event.preventDefault()
      let targetName = event.currentTarget.dataset.columnName

      if (event.currentTarget.classList.contains("selected")) {
        this.targetNames.delete(targetName)
        event.currentTarget.classList.remove("selected")
      } else {
        this.targetNames.add(targetName)
        event.currentTarget.classList.add("selected")
      }
    }

    nextStep() {
        this.index += 1
        this.renderSteps()
        this.renderProgressBar()
    }

    previousStep() {
        this.index -= 1
        this.renderSteps()
        this.renderProgressBar()
    }
}
