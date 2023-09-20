import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = ["input", "title", "titleContainer"]

  initialize() {
    this.inputTarget.addEventListener("focusout", (e) => {
      this.titleTarget.innerHTML = e.target.value
      this.toggleEditor()
    })

    // blur input on enter
    this.inputTarget.addEventListener("keydown", (e) => {
      if(e.key == "Enter") {
        this.inputTarget.blur()
      }
    })
  }

  toggleEditor() {
    if(this.inputTarget.style.display == "none") {
      this.inputTarget.style.display = "block"
      this.titleContainerTarget.style.display = "none"
      this.inputTarget.focus()
    } else {
      this.inputTarget.style.display = "none"
      this.titleContainerTarget.style.display = "flex"
    }
  }
}
