import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  startSearch() {
    this.element.querySelector(".dropdown-menu").classList.add("show");
  }

  endSearch() {
    this.element.querySelector(".dropdown-menu").classList.remove("show");
  }

  // Replace the src attribute of the turbo-frame
  // 250ms after the input has changed value. If another
  // change happens before the 250ms, the previous request is not sent.
  searchDebounced(e) {
    if (this.searchTimeout) {
      clearTimeout(this.searchTimeout);
    }

    const id = this.element.dataset.searchFrameId;
    const url = `${this.element.dataset.searchFrameUrl}${encodeURIComponent(
      e.currentTarget.value,
    )}`;

    this.searchTimeout = setTimeout(() => {
      this.search(id, url);
    }, 250);
  }

  search(id, url) {
    this.element.querySelector(`turbo-frame[id=${id}]`).src = url;
  }

  // Hide the dropdown if the user clicks outside of it.
  hideDropdown(e) {
    if (!this.element.contains(e.target)) {
      this.endSearch();
    }
  }
}
