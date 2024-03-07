import { Controller } from '@hotwired/stimulus'

export default class extends Controller {

  static targets = [
    'form',
  ]

  async submitRequest() {
    fetch(this.formTarget.action, {
      method: "POST",
      body: new FormData(this.formTarget),
    })
    .then(response => response.json())
    .then(rsp => console.log(rsp.rsp));

  }
}
