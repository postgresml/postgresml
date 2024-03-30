import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = ["input", "range"]

  onInputChange(e) {
    this.rangeTarget.value = e.currentTarget.value
  }

  onRangeChange(e) {
    this.inputTarget.value = e.currentTarget.value
  }
}
