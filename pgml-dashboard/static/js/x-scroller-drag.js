import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {
    isDown = false;
    startX;
    scrollLeft;

    static targets = [
        "slider"
    ]
    
    // TODO: Fix firefox highlight on grab.
    grab(e) {
        this.isDown = true;
        this.startX = e.pageX - this.sliderTarget.offsetLeft;
        this.scrollLeft = this.sliderTarget.scrollLeft;
    }

    leave() {
        this.isDown = false;
    }

    release() {
        this.isDown = false;
    }

    move(e) {
        if(!this.isDown) return;
        e.preventDefault();
        const x = e.pageX - this.sliderTarget.offsetLeft;
        const difference = (x - this.startX);
        this.sliderTarget.scrollLeft = this.scrollLeft - difference;
    }

}
