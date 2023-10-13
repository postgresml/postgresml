import { Controller } from '@hotwired/stimulus'

export default class extends Controller {

  static targets = [
    "range",
    "text",
    "group"
  ]

  static values = {
    bounds: Object,
    initial: Number
  }

  initialize() {
    this.textTarget.value = this.rangeTarget.value
  }

  updateText(e) {
    this.textTarget.value = e.target.value
    this.groupTarget.dispatchEvent(new Event("rangeInput"))
  }

  updateRange(e) {
    if( e.target.value < this.boundsValue.min 
        || !e.target.value || !this.isNumeric(e.target.value)) {
      this.rangeTarget.value = this.boundsValue.min
      this.textTarget.value = this.boundsValue.min
    } else if( e.target.value > this.boundsValue.max) {
      this.rangeTarget.value = this.boundsValue.max
      this.textTarget.value = this.boundsValue.max
    } else {
      this.rangeTarget.value = e.target.value
    }

    this.groupTarget.dispatchEvent(new Event("rangeInput"))
  }

  isNumeric(n) {
    return !isNaN(parseFloat(n)) && isFinite(n);
  }

  reset() {
    this.rangeTarget.value = this.initialValue
    this.textTarget.value = this.initialValue
  }
}
