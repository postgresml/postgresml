import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["button", "link"];

  selectSwitchOption(e) {
    this.buttonTargets.forEach((target) => {
      target.classList.remove("active");
      target.ariaPressed = false;
    });

    e.currentTarget.classList.add("active");
    e.currentTarget.ariaPressed = true;

    if (this.hasLinkTarget) {
      this.linkTarget.click()
    }
  }
}
