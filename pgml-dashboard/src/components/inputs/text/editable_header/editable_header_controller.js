import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = ["input", "header"]

  initialize() {
    this.inputTarget.addEventListener("focusout", (e) => {
      this.headerTarget.innerHTML = e.target.value
      this.toggleEditor()
    })

    // blur input on enter
    this.inputTarget.addEventListener("keydown", (e) => {
      if(e.key == "Enter") {
        this.inputTarget.blur()
      }
    })
  }

  toggleEditor(e) {
    // dont toggle if click inside input
    if( e && this.inputTarget.contains(e.target)) {
      return 
    }

    if(this.inputTarget.style.display == "none") {
      this.inputTarget.style.display = "block"
      this.headerTarget.style.display = "none"
      this.inputTarget.focus()
    } else {
      this.inputTarget.style.display = "none"
      this.headerTarget.style.display = "flex"
    }
  }
}
