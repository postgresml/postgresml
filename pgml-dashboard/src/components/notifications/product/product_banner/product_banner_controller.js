import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static values = {
    modal: String,
    showModal: Boolean
  };


  initialize() {}

  connect() {
    if (this.showModalValue) {
      const myModal = new bootstrap.Modal(document.getElementById(this.modalValue), {})
      myModal.show();
    }
  }

  updateModalCookie() {
    console.log("updating cookie")
  }

  disconnect() {}
}