import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["input", "range"];

  onInputInput(e) {
    const value = parseInt(e.currentTarget.value);

    if (isNaN(value)) {
      e.currentTarget.invalid = true;
    } else {
      this.rangeTarget.value = e.currentTarget.value;
      e.currentTarget.invalid = false;
    }
  }

  onRangeInput(e) {
    this.inputTarget.value = e.currentTarget.value;
  }
}
