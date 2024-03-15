import {
  Controller
} from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    'searchTrigger',
  ]

  connect() {
    this.target = document.getElementById("search");
    this.searchInput = document.getElementById("search-input");
    this.searchFrame = document.getElementById("search-results")

    this.target.addEventListener('shown.bs.modal', this.focusSearchInput)
    this.target.addEventListener('hidden.bs.modal', this.updateSearch)
    this.searchInput.addEventListener('input', (e) => this.search(e))

    this.timer;

    document.addEventListener("click", this.handle_search_click);
  }

  handle_search_click(e) {
    const target = e.target.closest(".search-result");
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

  search(e) {
    clearTimeout(this.timer);
    const query = e.currentTarget.value
    this.timer = setTimeout(() => {
      this.searchFrame.src = `/search?query=${query}`
    }, 250);
  }

  focusSearchInput = () => {
    this.searchInput.focus()
    this.searchTriggerTarget.blur()
  }

  updateSearch = () => {
    this.searchTriggerTarget.value = this.searchInput.value
  }

  openSearch = (e) => {
    new bootstrap.Modal(this.target).show()
    this.searchInput.value = e.currentTarget.value
  }

  disconnect() {
    this.searchTriggerTarget.removeEventListener('shown.bs.modal', this.focusSearchInput)
    this.searchTriggerTarget.removeEventListener('hidden.bs.modal', this.updateSearch)
    document.removeEventListener("click", this.handle_search_click);
  }
}
