import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["button"];

  onClick(e) {
    this.buttonTargets.forEach((target) => {
      target.classList.remove("active");
      target.ariaPressed = false;
      target.querySelector("input").checked = false;
    });

    e.currentTarget.classList.add("active");
    e.currentTarget.ariaPressed = true;

    const input = e.currentTarget.querySelector("input");

    input.checked = true;
    input.dispatchEvent(new Event("change"));
  }
}
