import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["link"];

  // When page reloads we need to set the left nav to the current window
  // location since left nave is turbo permanent. Trigger this on event
  // rather than on connect since on connect() will fire prior to backend
  // redirects.
  connect() {
    this.callback = () => {
      this.setLeftNavToLocation();
    };

    document.addEventListener("turbo:load", this.callback);
  }

  // Find link element in the left nav that matches the current window
  // location and set to active
  setLeftNavToLocation() {
    this.removeAllActive();

    let tab = this.findTab();
    if (tab) {
      tab.classList.add("active");
    }
  }

  // Helper function to quickly remove all state styling
  removeAllActive() {
    for (let i = 0; i < this.linkTargets.length; i++) {
      this.linkTargets[i].classList.remove("active");
    }
  }

  // Recursive function to find the tab that matches the current window
  findTab(level = 1, tag = "a[href='/']") {
    let element = this.element.querySelectorAll(tag);
    if (element.length == 1) {
      return element[0];
    } else {
      let path_vec = window.location.pathname.split("/");
      if (level > path_vec.length) {
        return;
      }

      let path = path_vec.slice(0, level).join("/");
      let tag = 'a[href="' + path + '"]';

      return this.findTab(level + 1, tag);
    }
  }

  // Remove event listener when controller is disconnected
  disconnect() {
    document.removeEventListener("turbo:load", this.callback);
  }
}
