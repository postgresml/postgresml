import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["input", "range", "unit"];

  onInputInput(e) {
    const value = parseInt(e.currentTarget.value);

    if (isNaN(value)) {
      e.currentTarget.invalid = true;
    } else {
      this.rangeTarget.value = e.currentTarget.value;
      e.currentTarget.invalid = false;
    }
  }

  onInputFocusIn(e) {
    if (this.hasUnitTarget) {
      this.unitTarget.classList.add("focused");
    }
  }

  onInputBlur(e) {
    if (this.hasUnitTarget) {
      this.unitTarget.classList.remove("focused");
    }
  }

  onUnitClick(e) {
    this.inputTarget.focus();
  }

  onRangeInput(e) {
    this.inputTarget.value = e.currentTarget.value;
  }
}
