import {
    Controller
} from '@hotwired/stimulus';

export default class extends Controller {
  connect() {
    const spy = new bootstrap.ScrollSpy(document.body, {
      target: '#toc-nav',
      smoothScroll: true,
      rootMargin: '-10% 0% -50% 0%',
      threshold: [1],
    });
    document.body.addEventListener('activate.bs.scrollspy', function(e) {

    });
  }

  clicked(e) {
    console.log('clicked', this, e);
    let href = e.target.attributes.href.nodeValue;
    if (href) {
      if (href.startsWith("#")) {
        let hash = href.slice(1);
        if (window.location.hash != hash) {
          window.location.hash = hash;
        }
      }
    }
  }
}
