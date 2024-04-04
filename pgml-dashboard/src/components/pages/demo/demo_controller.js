import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["rgb"];

  selectRgb(e) {
    this.rgbTargets.forEach((e) => {
      const element = e.querySelector("[data-controller=cards-rgb]");
      const controller = this.application.getControllerForElementAndIdentifier(
        element,
        "cards-rgb",
      );

      controller.inactive();
    });

    const element = e.currentTarget.querySelector(
      "[data-controller=cards-rgb]",
    );
    const controller = this.application.getControllerForElementAndIdentifier(
      element,
      "cards-rgb",
    );

    controller.active();
  }
}
