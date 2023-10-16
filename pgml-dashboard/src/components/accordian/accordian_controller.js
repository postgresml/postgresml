import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  titleClick(e) {
    let target = e.currentTarget.getAttribute("data-value");
    let elements = document.getElementsByClassName("accordian-body");
    for (let i = 0; i < elements.length; i++) {
      elements[i].classList.remove("selected");
    }
    elements = document.getElementsByClassName("accordian-header");
    for (let i = 0; i < elements.length; i++) {
      elements[i].classList.remove("selected");
    }
    let element = document.querySelector(`[data-accordian-target="${target}"]`);
    element.classList.add("selected");
    e.currentTarget.classList.add("selected");
  }
}
