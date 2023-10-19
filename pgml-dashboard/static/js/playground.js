import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = ["test", "switch", "rangeOptions"]

  initialize() {
    this.errorH3 = new CustomEvent("error", { detail: "message passed through event h3" })
    this.clearH3 = new Event("clear")
    this.errorH2 = new CustomEvent("error", { detail: "message passed through event h2" })
    this.clearH2 = new Event("clear")

    this.rangeOptionsTarget.addEventListener("rangeInput", (e) => {
      console.log("rangeOptionsTarget input event value = ", e.detail)
    })
  }


  selectRow(event) {
   console.log('dataset: ', event.currentTarget.dataset)
  }

  addError(event) {
    document.getElementById("header-3").dispatchEvent(this.errorH3)
  }

  clearError(event) {
    document.getElementById("header-3").dispatchEvent(this.clearH3)
  }

  addErrorH2() {
    document.getElementById("header-2").dispatchEvent(this.errorH2)
  }

  clearErrorH2() {
    document.getElementById("header-2").dispatchEvent(this.clearH2)
  }

  testOnToggleSwitch(e) {
    console.log("run from switch on toggle: ", e.detail)
  }

  resetSwitch() {
    this.switchTarget.dispatchEvent(new Event("reset"))
  }

  resetOptionsRange() {
    this.rangeOptionsTarget.dispatchEvent(new Event("reset"))
  }

  logOptionsRange() {
    console.log("rangeOptionsTarget current value = ", this.rangeOptionsTarget.dataset.detail)
  }

}
