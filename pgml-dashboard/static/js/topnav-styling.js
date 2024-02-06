import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {
    static values = {
        altStyling: Boolean
      }

    initialize() {
        this.pinned_to_top = false;
    }
    
    connect() {
        if( !this.altStylingValue ) {
            this.act_when_scrolled();
            this.act_when_expanded();
        }
    }

    act_when_scrolled() {
        // check scroll position in initial render
        if( window.scrollY > 48) {
            this.pinned_to_top = true; 
            this.element.classList.add("pinned");
        }

        addEventListener("scroll", (event) => {
            if (window.scrollY > 48 && !this.pinned_to_top) {
                this.pinned_to_top = true;
                this.element.classList.add("pinned");
            }
            
            if (window.scrollY < 48 && this.pinned_to_top) {
                this.pinned_to_top = false;
                this.element.classList.remove("pinned");
            };
        })
    }

    // Applies a class when navbar is expanded, used in mobile view for adding background contrast.
    act_when_expanded() {
        addEventListener('show.bs.collapse', (e) => {
            if (e.target.id === 'navbarSupportedContent') {
                this.element.classList.add('navbar-expanded');
            }
        })
        addEventListener('hidden.bs.collapse', (e) => {
            if (e.target.id === 'navbarSupportedContent') {
                this.element.classList.remove('navbar-expanded');
            }
        })
    }
    
}
