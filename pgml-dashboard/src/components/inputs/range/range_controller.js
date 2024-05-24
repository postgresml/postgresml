import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["range", "line"];

  static values = {
    interpolationType: String,
    min: Number,
    max: Number,
    initial: Number,
  };

  static outlets = [];

  initialize() {}

  connect() {
    this.rangeTarget.value =
      this.interpolationTypeValue === "exponential"
        ? this.exponentialInterpolationSolveX(this.initialValue)
        : this.linearInterpolationSolveX(this.initialValue);
  }

  onGrab() {
    if (this.hasLineTarget) {
      this.lineTarget.classList.add("grab-brightness");
    }
  }

  onRelease() {
    if (this.hasLineTarget) {
      this.lineTarget.classList.remove("grab-brightness");
    }
  }

  updateSlider(e) {
    this.rangeTarget.value =
      this.interpolationTypeValue === "exponential"
        ? this.exponentialInterpolationSolveX(e.detail)
        : this.linearInterpolationSolveX(e.detail);
  }

  sliderMoved() {
    this.dispatch("sliderMoved", {
      detail:
        this.interpolationTypeValue === "exponential"
          ? this.exponentialInterpolation(this.rangeTarget.value)
          : this.linearInterpolation(this.rangeTarget.value),
    });
  }

  exponentialInterpolation(value) {
    if (value < 1) {
      return this.minValue;
    }

    let minValue = this.minValue > 1 ? this.minValue : 1;

    let pow = value / 100;
    let out = minValue * Math.pow(this.maxValue / minValue, pow);
    return parseInt(Number(out.toPrecision(3)));
  }

  exponentialInterpolationSolveX(value) {
    if (value < 1) {
      return this.linearInterpolationSolveX(value);
    }

    let minValue = this.minValue > 1 ? this.minValue : 1;

    let numerator = Math.log(value / minValue);
    let denominator = Math.log(this.maxValue / minValue);
    let out = (numerator / denominator) * 100;
    return parseInt(Number(out.toPrecision(3)));
  }

  linearInterpolation(value) {
    let out = (this.maxValue - this.minValue) * (value / 100) + this.minValue;
    return parseInt(Number(out.toPrecision(3)));
  }

  linearInterpolationSolveX(value) {
    let out = ((value - this.minValue) / (this.maxValue - this.minValue)) * 100;
    return parseInt(Number(out.toPrecision(3)));
  }

  disconnect() {}
}
