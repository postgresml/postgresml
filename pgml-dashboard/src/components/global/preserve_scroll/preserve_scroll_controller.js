import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static initialized = false;

  initialize() {
    if (this.constructor.initialized) return;

    window.addEventListener("turbo:before-visit", function (event) {
      localStorage.setItem("scrollpos", window.scrollY);
    });

    window.addEventListener("turbo:load", function (event) {
      const scrollpos = localStorage.getItem("scrollpos");
      if (scrollpos) {
        window.scrollTo(0, scrollpos);
      }
    });

    this.constructor.initialized = true;
  }
}
