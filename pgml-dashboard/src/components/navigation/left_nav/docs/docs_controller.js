import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["level1Container", "level1Link", "highLevels"];

  // After page update we reset scroll position of nave back to where it was
  connect() {
    let nav = document.getElementsByClassName("doc-leftnav");
    if (nav.length > 0) {
      let position = nav[0].getAttribute("data-scroll");
      nav[0].scrollTop = position;
    }
  }

  // trubo-frame permanent breakes bootstrap data attribute collapse for aria
  // so we manually controll collapse
  expand(e) {
    let aria = e.currentTarget.getAttribute("aria-expanded");
    let id = e.currentTarget.getAttribute("aria-controls");

    let bsCollapse = bootstrap.Collapse.getOrCreateInstance(
      document.getElementById(id),
    );
    if (aria === "true") {
      bsCollapse.hide();
      e.currentTarget.setAttribute("aria-expanded", "false");
    } else {
      bsCollapse.show();
      e.currentTarget.setAttribute("aria-expanded", "true");
    }
  }

  // Activly manage nav state for level 1 links
  onNavigateManageLevel1(e) {
    this.removeAllActive();

    let container = e.currentTarget.closest("div");
    container.classList.add("active");

    e.currentTarget.classList.add("active");

    this.preventScrollOnNav();
  }

  // Activly manage nav state for high level links
  onNavigateManageHighLevels(e) {
    this.removeAllActive();

    let container = e.currentTarget.closest('div[data-level="1"]');
    let menu = container.querySelector(".menu-item");
    let link = menu.querySelector(".doc-left-nav-level1-link-container");

    link.classList.add("active");

    e.currentTarget.classList.add("purple");

    this.preventScrollOnNav();
  }

  // trubo-frame permanent scrolles nav to top on navigation so we capture the scrroll position prior
  // to updating the page so after we can set the scroll position back to where it was
  preventScrollOnNav() {
    let nav = document.getElementsByClassName("doc-leftnav");
    if (nav.length > 0) {
      let position = nav[0].scrollTop;
      nav[0].setAttribute("data-scroll", position);
    }
  }

  // Helper function to quickly remove all state styling
  removeAllActive() {
    for (let i = 0; i < this.highLevelsTargets.length; i++) {
      this.highLevelsTargets[i].classList.remove("purple");
    }

    for (let i = 0; i < this.level1ContainerTargets.length; i++) {
      this.level1ContainerTargets[i].classList.remove("active");
    }

    for (let i = 0; i < this.level1LinkTargets.length; i++) {
      this.level1LinkTargets[i].classList.remove("active");
    }
  }
}
