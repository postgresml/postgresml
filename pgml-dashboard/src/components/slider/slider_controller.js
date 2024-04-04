import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["item", "container", "indicatorItem"];

  static values = {
    index: Number,
    identifier: Number,
  };

  connect() {
    this.containerWidth = this.element.offsetWidth;
    this.itemWidth = this.itemTargets[0].offsetWidth;
    this.item0_offset = (this.containerWidth - this.itemWidth) / 2;

    // activate desired index
    this.active = this.indexValue;
    this.shift(this.indexValue);
  }

  // Mouse scroll event for left right scroll to change card
  scrollCheck(e) {
    let dx = e.deltaX;
    this.now = new Date();
    if (
      this.lastTimeScroll === undefined ||
      this.now - this.lastTimeScroll >= 400
    ) {
      this.lastTimeScroll = new Date();
      if (dx > 6 && this.active < this.itemTargets.length - 1) {
        this.shift(this.active + 1);
      } else if (dx < -6 && this.active > 0) {
        this.shift(this.active - 1);
      }
    }
  }

  // Monitor start touch swipe event for left right swipe to change card for mobile.
  startSwipe(e) {
    this.startX = e.touches[0].pageX;
  }

  // Monitor end touch swipe event for left right swipe to change card for mobile.
  endSwipe(e) {
    let dx = this.swipeDistance;
    if (dx < 30 && this.active < this.itemTargets.length - 1) {
      this.shift(this.active + 1);
    } else if (dx > -30 && this.active > 0) {
      this.shift(this.active - 1);
    }
  }

  // Measure touchscreen swipe distance
  swipeMove(e) {
    this.swipeDistance = e.touches[0].pageX - this.startX;
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
    this.changePagination(this.active, this.active);
  }

  changePagination(current, next) {
    let event = new CustomEvent("paginateNext", {
      detail: {
        current: current,
        next: next,
        identifier: this.identifierValue,
      },
    });
    window.dispatchEvent(event);
  }

  disconnect() {}
}
