import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["row"];

  selectRow(event) {
    this.rowTargets.forEach((row) => row.classList.remove("active"));
    event.currentTarget.classList.add("active");
  }
}
