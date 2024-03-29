import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["paginationItem"];

  static values = {
    index: Number,
    activeClass: String,
  };

  connect() {
    this.dispatch("connected", {});
  }

  changePagination(e) {
    this.shift(e.detail.current, e.detail.next);
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
      detail: { index: e.params.index },
    });
  }

  pause() {
    document
      .getElementsByClassName(this.activeClassValue)[0]
      .classList.add("pagination-timer-pause");
  }

  resume() {
    document
      .getElementsByClassName(this.activeClassValue)[0]
      .classList.remove("pagination-timer-pause");
  }
}
