import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["stateReference"];
  expand() {
    if (!this.stateReferenceTarget.classList.contains("show")) {
      const elements = this.element.getElementsByClassName("leftnav-collapse");
      for (const element of elements) {
        bootstrap.Collapse.getOrCreateInstance(element).show();
      }
    }
  }

  collapse() {
    if (this.stateReferenceTarget.classList.contains("show")) {
      const elements = this.element.getElementsByClassName("leftnav-collapse");
      for (const element of elements) {
        bootstrap.Collapse.getOrCreateInstance(element, {
          toggle: false,
        }).hide();
      }
    }
  }

  checkIfHover() {
    this.element.matches(":hover") ? this.expand() : this.collapse();
  }
}
