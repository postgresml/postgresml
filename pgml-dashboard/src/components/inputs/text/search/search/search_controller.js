import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  startSearch() {
    this.element.querySelector(".dropdown-menu").classList.add("show");
  }

  endSearch() {
    this.element.querySelector(".dropdown-menu").classList.remove("show");
  }

  search(e) {
    const id = this.element.dataset.searchFrameId;
    const url = `${this.element.dataset.searchFrameUrl}${encodeURIComponent(
      e.currentTarget.value,
    )}`;
    this.element.querySelector(`turbo-frame[id=${id}]`).src = url;
  }
}
