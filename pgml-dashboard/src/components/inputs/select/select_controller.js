import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["input", "value"];

  choose(e) {
    this.setValue(e.target.innerHTML);
  }

  // Choose value from dropdown option data-value attribute.
  // This separates the display value from the value passed to the input element.
  chooseValue(e) {
    this.inputTarget.value = e.currentTarget.dataset.value;
    this.valueTarget.innerHTML = e.currentTarget.innerHTML;
    this.inputTarget.dispatchEvent(new Event("change"));
  }

  resetSelect() {
    this.setValue(this.element.dataset.initial);
  }

  setValue(value) {
    this.inputTarget.value = value;
    this.valueTarget.innerHTML = value;
    this.inputTarget.dispatchEvent(new Event("change"));
  }
}
