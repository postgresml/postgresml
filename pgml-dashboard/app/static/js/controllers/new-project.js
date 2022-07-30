import { Controller } from "@hotwired/stimulus"
import { renderModel, renderDistribution, renderCorrelation, renderOutliers } from "@postgresml/main"

export default class extends Controller {
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
        this.index = 0
        this.targetNames = new Set()
        this.algorithmNames = new Set()

        this.checkDataSourceDebounced = _.debounce(this.checkDataSource, 250)
        this.checkProjectNameDebounced = _.debounce(this.checkProjectName, 250)
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
      // Let's get stuck on 97 just like Windows Update... ;)
      if (this.progressBarInterval && this.progressBarProgress >= 95)
        clearInterval(this.progressBarInterval)

      this.progressBarProgress += 2
      const progress = Math.min(100, this.progressBarProgress)

      this.progressBarTarget.style = `width: ${progress > 0 ? progress : "auto"}%;`
      this.progressBarAmountTarget.innerHTML = `${progress}%`
    }

    checkDataSource(event) {
        let tableName = event.target.value

        myFetch(`/api/tables/?table_name=${tableName}`)
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

      myFetch(`/api/projects/?name=${projectName}`)
      .then(res => res.json())
      .then(json => {
        if (json.results.length > 0) {
          this.projectName = null
        } else {
          this.projectName = projectName
        }

        this.renderProjectStatus()
      })
    }

    selectTask(event) {
      event.preventDefault()

      this.taskName = event.currentTarget.dataset.task

      if (this.taskName  === "regression") {
        this.algorithmListClassificationTarget.classList.add("hidden")
        this.algorithmListRegressionTarget.classList.remove("hidden")
      } else if (this.taskName  == "classification") {
        this.algorithmListClassificationTarget.classList.remove("hidden")
        this.algorithmListRegressionTarget.classList.add("hidden")
      }

      this.taskTargets.forEach(task => {
        task.classList.remove("selected")
      })

      event.currentTarget.classList.add("selected")
      this.taskNameNextTarget.disabled = false
    }

    selectAlgorithm(event) {
      event.preventDefault()

      let algorithmName = event.currentTarget.dataset.algorithm

      if (event.currentTarget.classList.contains("selected")) {
        event.currentTarget.classList.remove("selected")
        this.algorithmNames.delete(algorithmName)
      } else {
        event.currentTarget.classList.add("selected")
        this.algorithmNames.add(algorithmName)
      }

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
        myFetch(`/api/tables/sample/?table_name=${this.tableName}`)
        .then(res => res.text())
        .then(html => this.sampleTarget.innerHTML = html)
    }

    renderTarget() {
      myFetch(`/api/tables/columns/?table_name=${this.tableName}`)
      .then(res => res.text())
      .then(html => this.trainingLabelTarget.innerHTML = html)
    }

    renderAnalysisResult() {
      const snapshotData = this.projectData.models[0].snapshot

      console.log("Fetching analysis")
      myFetch(`/html/snapshots/analysis/?snapshot_id=${snapshotData.id}`)
      .then(res => res.text())
      .then(html => this.analysisResultTarget.innerHTML = html)
      .then(() => {
        // Render charts
        for (let name in snapshotData.columns) {
          const sample = JSON.parse(document.getElementById(name).textContent)
          renderDistribution(name, sample, snapshotData.analysis[`${name}_dip`])

          for (let target of snapshotData.y_column_name) {
            if (target === name)
              continue

            const targetSample = JSON.parse(document.getElementById(target).textContent)
            renderCorrelation(name, target, sample, targetSample)
          }
        }

        for (let target of snapshotData.y_column_name) {
          const targetSample = JSON.parse(document.getElementById(target).textContent)
          renderOutliers(target, targetSample, snapshotData.analysis[`${target}_stddev`])
        }

        this.progressBarProgress = 100
        this.renderProgressBar()

        setTimeout(this.nextStep.bind(this), 1000)
      })
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

      if (this.targetNames.size > 0)
        this.analysisNextTarget.disabled = false
      else
        this.analysisNextTarget.disabled = true
    }

    createSnapshot(event) {
      event.preventDefault()

      // Train a linear algorithm by default
      this.algorithmNames.add("linear")

      this.nextStep()

      // Start the progress bar :)
      this.progressBarProgress = 2
      this.progressBarInterval = setInterval(this.renderProgressBar.bind(this), 850)

      this.createProject(event, false, () => {
        this.index += 1 // Skip error page
        this.renderAnalysisResult()
        this.algorithmNames.delete("linear")
      })
    }

    createProject(event, redirect = true, callback = null) {
      event.preventDefault()

      const request = {
        "project_name": this.projectName,
        "task": this.taskName,
        "algorithms": Array.from(this.algorithmNames),
        "relation_name": this.tableName,
        "y_column_name": Array.from(this.targetNames),
      }

      if (redirect)
        this.createLoader()

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
            this.projectErrorTarget.innerHTML = json.error
            this.nextStep()
          })
          throw Error(`Failed to train project: ${json.error}`)
        }
      })
      .then(json => {
        this.projectData = json

        if (redirect)
          window.location.assign(`/${window.urlPrefix}/projects/${json.id}`);

        if (callback)
          callback()
      })
    }

    createLoader() {
      let element = document.createElement("div")
      element.innerHTML = `
        <div id="loader">
          <div class="loader"></div>
        </div>
      `;
      document.body.appendChild(element)
    }

    nextStep() {
        this.index += 1
        this.renderSteps()
    }

    previousStep() {
        this.index -= 1
        this.renderSteps()
    }

    restart() {
      this.index = 0
      this.renderSteps()
    }
}
