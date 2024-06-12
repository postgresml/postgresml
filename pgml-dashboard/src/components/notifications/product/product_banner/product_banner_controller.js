import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static values = {
    modal: String,
    showModal: Boolean,
    notificationId: String,
  };

  initialize() {}

  connect() {
    this.on_page_loaded = () => {
      if (this.showModalValue) {
        document
          .getElementById(this.modalValue)
          .dispatchEvent(new CustomEvent("show"));
      }
    };

    window.addEventListener("load", this.on_page_loaded);
  }

  updateModalCookie() {
    fetch(
      "/dashboard/notifications/product/modal/remove_modal?id=" +
        this.notificationIdValue,
      {},
    );
  }

  closeModal(e) {
    e.preventDefault();
    document
      .getElementById(this.modalValue)
      .dispatchEvent(new CustomEvent("hide"));
    Turbo.visit(e.target.href);
  }

  disconnect() {
    window.removeEventListener("load", this.on_page_loaded);
  }
}
