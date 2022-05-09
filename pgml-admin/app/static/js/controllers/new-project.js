import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
    static targets = ["step", "progressBar", "progressBarAmount", "sample", "tableStatus", "dataSourceNext", "projectStatus", "projectNameNext", "trainingLabel"];

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
            else {
                this.tableName = null
                this.sampleTarget.innerHTML = ""
                this.trainingLabelTarget.innerHTML = ""
              }
            this.renderTableStatus()
        })
        .catch(err => {
            this.tableName = null
            this.renderTableStatus()
        })
    }

    checkProjectName(event) {
      let projectName = event.target.value

      fetch(`/api/projects/?name=${projectName}`)
      .then(res => res.json())
      .then(json => {
        if (json.length > 0) {
          this.projectName = null
        } else {
          this.projectName = projectName
        }

        this.renderProjectStatus()
      })
    }

    renderTableStatus() {
        if (this.tableName) {
            this.tableStatusTarget.innerHTML = "done"
            this.tableStatusTarget.classList.add("ok")
            this.tableStatusTarget.classList.remove("error")
            this.dataSourceNextTarget.disabled = false
        } else {
            this.tableStatusTarget.innerHTML = "close"
            this.tableStatusTarget.classList.add("error")
            this.tableStatusTarget.classList.remove("ok")
            this.dataSourceNextTarget.disabled = true
        }
        
    }

    renderProjectStatus() {
      if (this.projectName) {
            this.projectStatusTarget.innerHTML = "done"
            this.projectStatusTarget.classList.add("ok")
            this.projectStatusTarget.classList.remove("error")
            this.projectNameNextTarget.disabled = false
        } else {
            this.projectStatusTarget.innerHTML = "close"
            this.projectStatusTarget.classList.add("error")
            this.projectStatusTarget.classList.remove("ok")
            this.projectNameNextTarget.disabled = true
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
