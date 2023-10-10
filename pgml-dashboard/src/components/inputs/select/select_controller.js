import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = ["input", "value"]

  choose(e) {
    this.inputTarget.value = e.target.innerHTML
    this.valueTarget.innerHTML = e.target.innerHTML
    this.inputTarget.dispatchEvent(new Event('change'))
  }
}
