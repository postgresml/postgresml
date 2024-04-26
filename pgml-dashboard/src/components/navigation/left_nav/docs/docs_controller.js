import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["level1Container", "level1Link", "highLevels", "leftNav"];

  // After page update we reset scroll position of nav back to where it
  // was and ensure left nave and window location match.
  connect() {
    let nav = document.getElementsByClassName("doc-leftnav");
    if (nav.length > 0) {
      let position = nav[0].getAttribute("data-scroll");
      nav[0].scrollTop = position;
    }

    this.callback = () => {
      this.setNavToLocation();
    };

    document.addEventListener("turbo:load", this.callback);
  }

  // The active tags should always be set to the current page location
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

  expandSubmenuIfExists(containerEl) {
    const controllerEl = containerEl.querySelector(
      "[data-action='click->navigation-left-nav-docs#toggle']",
    );
    controllerEl ? this.expand(controllerEl) : null;
  }

  // Finds all parent submenus this element is in and expands them. Takes
  // the element containing the current level
  expandAllParents(element) {
    let level = element.getAttribute("data-level");

    this.expandSubmenuIfExists(element);
    if (level > 1) {
      let next = "div[data-level='" + (parseInt(level) - 1) + "']";
      this.expandAllParents(element.closest(next));
    }
  }

  // turbo-frame-permanent breaks bootstrap data attribute collapse for aria
  // so we manually control collapse
  toggle(event) {
    let aria = event.currentTarget.getAttribute("aria-expanded");

    if (aria === "true") {
      this.collapse(event.currentTarget);
    } else {
      this.expand(event.currentTarget);
    }
  }

  // Expands the submenu, takes submenu control element.
  expand(element) {
    let id = element.getAttribute("aria-controls");
    let aria = element.getAttribute("aria-expanded");

    if (aria === "false") {
      let bsCollapse = bootstrap.Collapse.getOrCreateInstance(
        document.getElementById(id),
      );
      bsCollapse.show();
      element.setAttribute("aria-expanded", "true");
    }
  }

  // Collapses the submenu, takes submenu control element.
  collapse(element) {
    let id = element.getAttribute("aria-controls");
    let aria = element.getAttribute("aria-expanded");

    if (aria === "true") {
      let bsCollapse = bootstrap.Collapse.getOrCreateInstance(
        document.getElementById(id),
      );
      bsCollapse.hide();
      element.setAttribute("aria-expanded", "false");
    }
  }

  // Actively manage nav state for high level links.
  setHighLevelLeftNav(element) {
    this.removeAllActive();

    const parentContainer = element.closest('div[data-level="1"]');
    const parentMenu = parentContainer.querySelector(".menu-item");
    const parentLink = parentMenu.querySelector(
      ".doc-left-nav-level1-link-container",
    );

    parentLink.classList.add("active");
    element.classList.add("purple");

    const container = element.parentElement;
    this.expandSubmenuIfExists(container);

    const levelEl = container.closest("div[data-level]");
    this.expandAllParents(levelEl);

    this.preventScrollOnNav();
  }

  // Actively manage nav state for level 1 links
  setLevel1LeftNav(element) {
    this.removeAllActive();

    const container = element.closest("div");
    container.classList.add("active");

    element.classList.add("active");

    this.expandSubmenuIfExists(container);

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
    if (this.hasLeftNavTarget) {
      let position = this.leftNavTarget.scrollTop;
      this.leftNavTarget.setAttribute("data-scroll", position);
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

  disconnect() {
    document.removeEventListener("turbo:load", this.callback);
  }
}
