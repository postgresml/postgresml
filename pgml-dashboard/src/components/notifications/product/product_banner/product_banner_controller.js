import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static values = {
    modal: String,
    notificationId: String,
  };

  updateModalCookie() {
    fetch(
      "/dashboard/notifications/product/modal/remove_modal?id=" +
        this.notificationIdValue,
      {},
    );
  }

  followModalLink(e) {
    e.preventDefault();
    this.hideModal();
    Turbo.visit(e.target.href);
  }

  hideModal() {
    document
      .getElementById(this.modalValue)
      .dispatchEvent(new CustomEvent("hide"));
  }

  showModal() {
    document
      .getElementById(this.modalValue)
      .dispatchEvent(new CustomEvent("show"));
  }
}
