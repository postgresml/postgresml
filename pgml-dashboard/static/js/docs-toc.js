import {
    Controller
} from '@hotwired/stimulus';

export default class extends Controller {
  connect() {
    this.scrollSpyAppend();
  }

  scrollSpyAppend() {
    const spy = new bootstrap.ScrollSpy(document.body, {
      target: '#toc-nav',
      smoothScroll: true,
      rootMargin: '-10% 0% -50% 0%',
      threshold: [1],
    })
  }
}
