import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {
    static targets = [
        'searchInput',
        'searchModal',
        'searchTrigger',
        'searchFrame',
    ]

    connect() {
        this.searchModalTarget.addEventListener('shown.bs.modal', this.focusSearchInput)
        this.searchModalTarget.addEventListener('hidden.bs.modal', this.updateSearch)
    }

    search(e) {
        const query = e.currentTarget.value
        this.searchFrameTarget.src = `/docs/search?query=${query}`
    }

    focusSearchInput = (e) => {
        this.searchInputTarget.focus()
        this.searchTriggerTarget.blur()
    }

    updateSearch = () => {
      this.searchTriggerTarget.value = this.searchInputTarget.value
    }

    openSearch = (e) => {
      new bootstrap.Modal(this.searchModalTarget).show()
      this.searchInputTarget.value = e.currentTarget.value
    }

    disconnect() {
        this.searchTriggerTarget.removeEventListener('shown.bs.modal', this.focusSearchInput)
        this.searchTriggerTarget.removeEventListener('hidden.bs.modal', this.updateSearch)
    }
}
