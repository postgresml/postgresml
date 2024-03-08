import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["toggle", "toggleText", "toggleIcon"];

  static values = {
    left: String,
    right: String,
    initial: String,
    leftIcon: String,
    rightIcon: String,
  };

  toggle() {
    if (this.toggleTarget.classList.contains("right")) {
      this.onToggleLeft();
    } else {
      this.onToggleRight();
    }
  }

  onToggleLeft() {
    this.toggleTarget.classList.remove("right");
    this.toggleTarget.classList.add("left");
    this.toggleTextTarget.innerHTML = this.leftValue;
    this.toggleIconTarget.innerHTML = this.leftIconValue;
    this.element.dispatchEvent(
      new CustomEvent("toggle", { detail: this.leftValue }),
    );
  }

  onToggleRight() {
    this.toggleTarget.classList.remove("left");
    this.toggleTarget.classList.add("right");
    this.toggleTextTarget.innerHTML = this.rightValue;
    this.toggleIconTarget.innerHTML = this.rightIconValue;
    this.element.dispatchEvent(
      new CustomEvent("toggle", { detail: this.rightValue }),
    );
  }

  reset() {
    if (this.initialValue == "left") {
      console.log("toggling left");
      this.onToggleLeft();
    } else {
      console.log("toggling right");
      this.onToggleRight();
    }
  }
}
