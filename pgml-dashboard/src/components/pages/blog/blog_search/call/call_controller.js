import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static targets = ["searchFrame", "searchInput", "tagLink", "removeTags"];

  static classes = ["selected"];

  static outlets = [];

  connect() {
    this.timer;
    this.tags = "";

    document.addEventListener("click", this.handle_search_click);
  }

  handle_search_click(e) {
    const target = e.target.closest(".blog-search-result");
    if (target) {
      const resultIndex = target.getAttribute("data-result-index");
      const searchId = target.getAttribute("data-search-id");
      const formData = new FormData();
      formData.append("search_id", searchId);
      formData.append("clicked", resultIndex);
      fetch('/search_event', {
        method: 'POST',
        body: formData,
      });
    }
  }

  search() {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.searchFrameTarget.src = `/search_blog?query=${this.searchInputTarget.value}&tag=${this.tags}`;
    }, 250);
  }

  tag(e) {
    if (e.target.classList.contains(this.selectedClass)) {
      e.target.classList.remove(this.selectedClass);
      this.tags = "";
      this.removeTagsTarget.classList.add(this.selectedClass);
    } else {
      e.target.classList.add(this.selectedClass);
      this.tags = e.params.tag;
      this.removeTagsTarget.classList.remove(this.selectedClass);
    }

    for (let tag of this.tagLinkTargets) {
      if (tag != e.target) {
        tag.classList.remove(this.selectedClass);
      }
    }

    this.search();
  }

  removeTags() {
    for (let tag of this.tagLinkTargets) {
      tag.classList.remove(this.selectedClass);
    }

    this.removeTagsTarget.classList.add(this.selectedClass);

    this.tags = "";
    this.search();
  }

  disconnect() {
    document.removeEventListener("click", this.handle_search_click);
  }
}
