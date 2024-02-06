import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = []
  static outlets = []

  initialize() {
    console.log('Initialized navigation-left-nav-docs')
  }
  
  showScrollbar(e) {
    let element = e.target;

    element.classList.add('show-scroll');
    clearTimeout(this.timer);
    this.timer = setTimeout(function() {
      element.classList.remove('show-scroll');
    }, 100);  
  }
}