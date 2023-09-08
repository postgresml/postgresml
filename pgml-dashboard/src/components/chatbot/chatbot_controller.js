import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  initialize() {
    console.log('Initialized chatbot')
    window.addEventListener("keypress", function (e) {
      console.log(this.element);
    }.bind(this));
  }

  connect() {
  }

  disconnect() {}
}
