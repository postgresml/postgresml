import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = ["input", "value"]

  choose(e) {
    this.setValue(e.target.innerHTML)
  }
  
  resetSelect() {
    this.setValue(this.element.dataset.initial)
  }

  setValue(value) {
    this.inputTarget.value = value
    this.valueTarget.innerHTML = value
    this.inputTarget.dispatchEvent(new Event('change'))
  }
}
