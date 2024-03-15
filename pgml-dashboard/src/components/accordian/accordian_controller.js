import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  initialize() {
    this.bodies = document.getElementsByClassName("accordian-body");
    this.headers = document.getElementsByClassName("accordian-header");

    this.heights = new Map();
    for (let i = 0; i < this.bodies.length; i++) {
      this.heights.set(this.bodies[i], this.bodies[i].offsetHeight);
      if (i > 0) {
        this.bodies[i].style.maxHeight = "0px";
      } else {
        this.bodies[i].style.maxHeight = this.bodies[i].offsetHeight + "px";
      }
    }
  }

  titleClick(e) {
    let target = e.currentTarget.getAttribute("data-value");
    e.currentTarget.classList.add("selected");

    let body = document.querySelector(`[data-accordian-target="${target}"]`);
    body.classList.add("selected");
    body.style.maxHeight = this.heights.get(body) + "px";

    for (let i = 0; i < this.bodies.length; i++) {
      if (body != this.bodies[i]) {
        this.bodies[i].classList.remove("selected");
        this.bodies[i].style.maxHeight = "0px";
      }
      if (e.currentTarget != this.headers[i])
        this.headers[i].classList.remove("selected");
    }
  }
}
