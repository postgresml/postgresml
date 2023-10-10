import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = ["test"]

  initialize() {
    this.errorH3 = new CustomEvent("error", { detail: "message passed through event h3" })
    this.clearH3 = new Event("clear")
    this.errorH2 = new CustomEvent("error", { detail: "message passed through event h2" })
    this.clearH2 = new Event("clear")
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

}
