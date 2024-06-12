import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static values = {
    modal: String,
    showModal: Boolean,
    notificationId: String,
  };


  initialize() {}

  connect() {
    if (this.showModalValue) {
      const myModal = new bootstrap.Modal(document.getElementById(this.modalValue), {})
      myModal.show();
    }
  }

  updateModalCookie() {
    fetch("/dashboard/notifications/product/modal/remove_modal?id=" + this.notificationIdValue, {})
  }

  disconnect() {}
}