import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["paginationItem"];

  static values = {
    index: Number,
    activeClass: String,
    identifier: Number,
  };

  connect() {
    this.dispatch("connected", {
      detail: { identifier: this.identifierValue },
    });
  }

  changePagination(e) {
    if (e.detail.identifier == this.identifierValue) {
      this.shift(e.detail.current, e.detail.next);
    }
  }

  shift(current, next) {
    let items = this.paginationItemTargets;
    let currentItem = items[current];
    let nextItem = items[next];

    if (currentItem) {
      currentItem.classList.remove(this.activeClassValue);
      currentItem.style.width = "1rem";
    }
    if (nextItem) {
      nextItem.style.width = "4rem";
      nextItem.classList.add(this.activeClassValue);
    }
  }

  change(e) {
    this.dispatch("change", {
      detail: { index: e.params.index, identifier: this.identifierValue },
    });
  }

  pause(e) {
    if (e.detail.identifier == this.identifierValue) {
      document
        .getElementsByClassName(this.activeClassValue)[0]
        .classList.add("pagination-timer-pause");
    }
  }

  resume(e) {
    if (e.detail.identifier == this.identifierValue) {
      document
        .getElementsByClassName(this.activeClassValue)[0]
        .classList.remove("pagination-timer-pause");
    }
  }
}
