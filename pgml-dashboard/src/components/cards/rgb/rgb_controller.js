import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  // Activate this card (add RGB).
  active() {
    this.element
      .querySelector(".card")
      .classList.add("main-gradient-border-card-1");
  }

  // Deactivate this card (remove RGB).
  inactive() {
    this.element
      .querySelector(".card")
      .classList.remove("main-gradient-border-card-1");
  }
}
