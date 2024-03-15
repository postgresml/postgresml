import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["modal"];

  connect() {
    this.modal = new bootstrap.Modal(this.modalTarget);
  }

  show() {
    this.modal.show();
  }

  hide() {
    this.modal.hide();
  }
}
