import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["level1Container", "level1Link", "highLevels"];

  // After page update we reset scroll position of nav back to where it
  // was and ensure left nave and window location match.
  // Stimulus connect runs on every page load regardless of the element
  // being permanent or not.
  connect() {
    let nav = document.getElementsByClassName("doc-leftnav");
    if (nav.length > 0) {
      let position = nav[0].getAttribute("data-scroll");
      nav[0].scrollTop = position;
    }

    this.setNavToLocation();
  }

  // The active tags should always be set the current page location
  setNavToLocation() {
    const tag = "a[href='" + window.location.pathname + "']";

    let link = this.element.querySelectorAll(tag);
    if (link.length > 0) {
      if (
        link[0].getAttribute("data-navigation-left-nav-docs-target") ==
        "highLevels"
      ) {
        this.setHighLevelLeftNav(link[0]);
      } else {
        this.setLevel1LeftNav(link[0]);
      }
    }
  }

  // turbo-frame-permanent breaks bootstrap data attribute collapse for aria
  // so we manually control collapse
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

  // Actively manage nav state for high level links.
  setHighLevelLeftNav(element) {
    this.removeAllActive();

    let container = element.closest('div[data-level="1"]');
    let menu = container.querySelector(".menu-item");
    let link = menu.querySelector(".doc-left-nav-level1-link-container");

    link.classList.add("active");
    element.classList.add("purple");
    this.preventScrollOnNav();
  }

  // Actively manage nav state for level 1 links
  setLevel1LeftNav(element) {
    this.removeAllActive();

    let container = element.closest("div");
    container.classList.add("active");

    element.classList.add("active");

    this.preventScrollOnNav();
  }

  // Actions to take when nav link is clicked
  // currently just gets the scroll position before state change
  onNavigateManageLevel1() {
    this.preventScrollOnNav();
  }

  // Actions to take when nav link is clicked
  // currently just gets the scroll position before state change
  onNavigateManageHighLevels() {
    this.preventScrollOnNav();
  }

  // turbo-frame permanent scrolls nav to top on navigation so we capture the scroll position prior
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
