import {
    Controller
} from '@hotwired/stimulus';

export default class extends Controller {
  connect() {
    this.scrollSpyAppend();
  }

  scrollSpyAppend(e) {
    // intercept click event callback so we can set the url
    if( e && e.type == "click") {
      console.log("append click")
      this.clicked(e)
    }

    const spy = new bootstrap.ScrollSpy(document.body, {
      target: '#toc-nav',
      smoothScroll: true,
      rootMargin: '-10% 0% -50% 0%',
      threshold: [1],
    })
  }

  clicked(e) {
    
    console.log("clicked clicked")
    let href = e.target.attributes.href.nodeValue;
    if (href) {
      if (href.startsWith("#")) {
        let hash = href.slice(1);
        if (window.location.hash != hash) {
          window.location.hash = hash
        }
      }
    }
  }
}
