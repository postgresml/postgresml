import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["carousel", "carouselTimer", "template"];

  static values = {
    identifier: Number,
  };

  initialize() {
    this.paused = false;
    this.runtime = 0;
    this.times = 0;
  }

  connect() {
    // dont cycle carousel if it only hase one item.
    if (this.templateTargets.length > 1) {
      this.cycle();
    }
  }

  changeFeatured(next) {
    let current = this.carouselTarget.children[0];
    let nextItem = next.content.cloneNode(true);

    this.carouselTarget.appendChild(nextItem);

    if (current) {
      current.style.marginLeft = "-100%";
      setTimeout(() => {
        this.carouselTarget.removeChild(current);
      }, 700);
    }
  }

  Pause() {
    this.paused = true;
    let pause = new CustomEvent("paginatePause", {
      detail: { identifier: this.identifierValue },
    });
    window.dispatchEvent(pause);
  }

  Resume() {
    this.paused = false;
    let resume = new CustomEvent("paginateResume", {
      detail: { identifier: this.identifierValue },
    });
    window.dispatchEvent(resume);
  }

  cycle() {
    this.interval = setInterval(() => {
      // maintain paused state through entire loop
      let paused = this.paused;

      if (!paused && this.runtime % 5 == 0) {
        let currentIndex = this.times % this.templateTargets.length;
        let nextIndex = (this.times + 1) % this.templateTargets.length;

        this.changePagination(currentIndex, nextIndex);
        this.changeFeatured(this.templateTargets[nextIndex]);
        this.times++;
      }

      if (!paused) {
        this.runtime++;
      }
    }, 1000);
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

  disconnect() {
    clearInterval(this.interval);
  }
}
