import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = [
    "range",
    "text",
    "group",
    "line",
    "tick",
    "tickText",
    "smScreenText",
  ];

  static values = {
    bounds: Object,
    initial: Number,
  };

  initialize() {
    this.textTarget.value = this.rangeTarget.value;
    this.updateTicks(this.rangeTarget.value);
    this.updateTicksText(this.rangeTarget.value);
  }

  updateText(e) {
    this.textTarget.value = e.target.value;
    this.element.dataset.detail = e.target.value;
    this.groupTarget.dispatchEvent(
      new CustomEvent("rangeInput", { detail: e.target.value }),
    );
  }

  updateRange(e) {
    if (
      e.target.value < this.boundsValue.min ||
      !e.target.value ||
      !this.isNumeric(e.target.value)
    ) {
      this.rangeTarget.value = this.boundsValue.min;
      this.textTarget.value = this.boundsValue.min;
    } else if (e.target.value > this.boundsValue.max) {
      this.rangeTarget.value = this.boundsValue.max;
      this.textTarget.value = this.boundsValue.max;
    } else {
      this.rangeTarget.value = e.target.value;
    }

    this.element.dataset.detail = this.rangeTarget.value;
    this.groupTarget.dispatchEvent(
      new CustomEvent("rangeInput", { detail: this.rangeTarget.value }),
    );
  }

  isNumeric(n) {
    return !isNaN(parseFloat(n)) && isFinite(n);
  }

  reset() {
    this.rangeTarget.value = this.initialValue;
    this.textTarget.value = this.initialValue;
    this.updateTicks(this.initialValue);
    this.updateTicksText(this.initialValue);
    this.element.dataset.detail = this.initialValue;
    this.groupTarget.dispatchEvent(
      new CustomEvent("rangeInput", { detail: this.rangeTarget.value }),
    );
  }

  on_grab() {
    if (this.hasLineTarget) {
      this.lineTarget.classList.add("grab-brightness");
    }

    if (this.hasTickTarget) {
      this.tickTargets.forEach((tick, index) => {
        if (index < this.rangeTarget.value) {
          tick.classList.add("grab-brightness");
        } else {
          tick.classList.remove("grab-brightness");
        }
      });
    }
  }

  on_release() {
    if (this.hasLineTarget) {
      this.lineTarget.classList.remove("grab-brightness");
    }

    if (this.hasTickTarget) {
      this.tickTargets.forEach((tick, index) => {
        if (index < this.rangeTarget.value) {
          tick.classList.remove("grab-brightness");
        }
      });
    }
  }

  updateTicks(value) {
    if (!this.hasTickTarget) return;

    this.tickTargets.forEach((tick, index) => {
      if (index < value) {
        tick.classList.add("active-color");
      } else {
        tick.classList.remove("active-color");
      }
    });
  }

  updateTicksText(value) {
    if (this.hasTickTextTarget && this.hasSmScreenTextTarget) {
      this.tickTextTargets.forEach((tickText, index) => {
        if (index + 1 == value) {
          tickText.classList.add("active-color");
          this.smScreenTextTargets[index].style.display = "flex";
        } else {
          tickText.classList.remove("active-color");
          this.smScreenTextTargets[index].style.display = "none";
        }
      });
    }
  }

  updateTicksEventWrapper(e) {
    this.updateTicks(e.target.value);
  }

  updateTicksTextEventWrapper(e) {
    this.updateTicksText(e.target.value);
  }
}
