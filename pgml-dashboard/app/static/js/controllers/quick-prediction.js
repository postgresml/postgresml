import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = [
    "feature",
    "step",
    "prediction",
  ]

  initialize() {
    this.index = 0
  }

  nextStep() {
    this.index += 1
    this.renderSteps()
  }

  prevStep() {
    this.index -= 1
    this.renderSteps()
  }

  renderSteps() {
    this.stepTargets.forEach((element, index) => {
      if (this.index !== index) {
        element.classList.add("hidden")
      } else {
        element.classList.remove("hidden")
      }
    })
  }

  predict(event) {
    const inputs = []

    this.featureTargets.forEach(target => {
      const name = target.getAttribute("name")
      const value = target.value

      inputs.push(Number(value))
    })

    const modelId = event.currentTarget.dataset.modelId

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
        element.innerHTML = json.predictions[index]
      })
      this.nextStep()
    })
  }
}
