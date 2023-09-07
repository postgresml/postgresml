import { Controller } from '@hotwired/stimulus'

export default class extends Controller {

  static targets = [
    "range",
    "text",
  ]

  static values = {
    bounds: Object
  }

  initialize() {
    this.textTarget.value = this.rangeTarget.value
  }

  updateText(e) {
    this.textTarget.value = e.target.value
  }

  updateRange(e) {
    if( e.target.value < this.boundsValue.min 
        || !e.target.value || !this.isNumeric(e.target.value)) {
      this.rangeTarget.value = this.boundsValue.min
      this.textTarget.value = this.boundsValue.min
      return 
    }

    if( e.target.value > this.boundsValue.max) {
      this.rangeTarget.value = this.boundsValue.max
      this.textTarget.value = this.boundsValue.max
      return
    }

    this.rangeTarget.value = e.target.value
  }

  isNumeric(n) {
    return !isNaN(parseFloat(n)) && isFinite(n);
  }
}
