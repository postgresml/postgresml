import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["input", "header", "error"];

  focusout(e) {
    this.headerTarget.innerHTML = e.target.value;
    this.toggleEditor();
  }

  blur() {
    this.inputTarget.blur();
  }

  toggleEditor(e) {
    // dont toggle if click inside input
    if (e && this.inputTarget.contains(e.target)) {
      return;
    }

    if (this.inputTarget.style.display == "none") {
      this.inputTarget.style.display = "block";
      this.headerTarget.style.display = "none";
      this.inputTarget.focus();
    } else {
      this.inputTarget.style.display = "none";
      this.headerTarget.style.display = "flex";
    }
  }

  error(e) {
    this.errorTarget.innerHTML = e.detail;
    this.errorTarget.style.display = "block";
    this.headerTarget.classList.add("error");
  }

  clear() {
    this.errorTarget.style.display = "none";
    this.headerTarget.classList.remove("error");
  }
}
