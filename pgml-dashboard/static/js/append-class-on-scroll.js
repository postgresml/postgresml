import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {
    initialize() {
        this.pinned = false;
    }
    
    connect() {
        addEventListener("scroll", (event) => {
            if (window.scrollY > 48 && !this.pinned) {
                this.pinned = true;
                this.element.classList.add("pinned");
            }
            
            if (window.scrollY < 48 && this.pinned) {
                this.pinned = false;
                this.element.classList.remove("pinned");
            };
        })
    }
}