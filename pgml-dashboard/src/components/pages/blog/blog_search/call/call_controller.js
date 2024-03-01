import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    'searchFrame',
    'searchInput'
  ]
  static outlets = []

  connect() {
    this.timer
  }

  search() {
    clearTimeout(this.timer)
    this.timer = setTimeout(() => {
      this.searchFrameTarget.src = `/search_blog?query=${this.searchInputTarget.value}`
      this.searchInputTarget.value = ""
    }, 250)
  }
}
