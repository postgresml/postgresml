import {
    Controller
} from '@hotwired/stimulus';

export default class extends Controller {
  setUrlFragment(e) {
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
