import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["textInput", "range"];
  static outlets = [];
  static values = {
    min: Number,
    max: Number,
  };

  connect() {
    this.updateDatasetValue();

    // when connected, update the slider and trigger the inputUpdated event
    this.textUpdated();
  }

  updateText(e) {
    if (e.detail >= this.minValue && e.detail <= this.maxValue) {
      this.removeErrorState();
      this.textInputTarget.value = e.detail;
      this.updateDatasetValue();
      this.inputUpdated();
    } else {
      this.applyErrorState();
    }
  }

  textUpdated() {
    let value = Number(this.textInputTarget.value);
    if (!value) {
      value = this.minValue;
      this.textInputTarget.value = value;
    }

    if (value > this.maxValue || value < this.minValue) {
      this.applyErrorState();
      value = value > this.maxValue ? this.maxValue : this.minValue;
      value = value < this.minValue ? this.minValue : value;
      this.dispatchToRange(value);
    } else {
      this.removeErrorState();
      this.dispatchToRange(value);
      this.updateDatasetValue();
      this.inputUpdated();
    }
  }

  // Tell anyone listening that the input has been updated
  inputUpdated() {
    this.dispatch("transmitValue", {});
  }

  // Attaches input value to the controller component
  updateDatasetValue() {
    this.element.dataset.value = this.textInputTarget.value;
  }

  applyErrorState() {
    this.element
      .getElementsByClassName("input-group")[0]
      .classList.add("error");
  }

  removeErrorState() {
    this.element
      .getElementsByClassName("input-group")[0]
      .classList.remove("error");
  }

  dispatchToRange(value) {
    if (this.hasRangeTarget) {
      this.rangeTarget.dispatchEvent(
        new CustomEvent("updateSlider", { detail: value }),
      );
    }
  }

  disconnect() {}
}
