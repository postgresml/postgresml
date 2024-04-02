import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  clickIcon() {
    this.element.querySelector("input").focus();
  }
}
