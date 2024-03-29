import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["item", "container", "indicatorItem"];

  connect() {
    this.containerWidth = this.element.offsetWidth;
    this.itemWidth = this.itemTargets[0].offsetWidth;
    this.item0_offset = (this.containerWidth - this.itemWidth) / 2;

    // activate middle card
    let middleItem = Math.floor(this.itemTargets.length / 2);
    this.active = middleItem;
    this.shift(middleItem);
  }

  scrollCheck(e) {
    let dx = e.deltaX;
    this.now = new Date();
    if (this.lastTime === undefined || this.now - this.lastTime >= 400) {
      this.lastTime = new Date();
      if (dx > 6 && this.active < this.itemTargets.length - 1) {
        this.shift(this.active + 1);
      } else if (dx < -6 && this.active > 0) {
        this.shift(this.active - 1);
      }
    }
  }

  next(e) {
    this.shift(e.params.index);
  }

  nextFromPagination(e) {
    this.shift(e.detail.index);
  }

  shift(index) {
    let current = this.active;
    this.active = index;
    for (let i = 0; i < this.itemTargets.length; i++) {
      this.disable(this.itemTargets[i]);
    }
    this.activate(this.itemTargets[index]);

    let shift = index * this.itemWidth;
    this.containerTarget.style.marginLeft = this.item0_offset - shift + "px";

    this.changePagination(current, index);
  }

  activate(item) {
    item.classList.remove("disabled");
    item.classList.add("active");
  }

  disable(item) {
    item.classList.remove("active");
    item.classList.add("disabled");
  }

  scroller(dx) {
    if (dx > 6 && this.active < this.itemTargets.length - 1) {
      this.shift(this.active + 1);
    } else if (dx < -6 && this.active > 0) {
      this.shift(this.active - 1);
    }
  }

  changePaginationInit() {
    this.changePagination(1, 1);
  }

  changePagination(current, next) {
    let event = new CustomEvent("paginateNext", {
      detail: { current: current, next: next },
    });
    window.dispatchEvent(event);
  }

  disconnect() {}
}
