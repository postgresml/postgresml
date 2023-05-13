// Gym controller.

import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {
    static targets = [
        'frame',
    ];

    click(event) {
        let href = event.currentTarget.dataset.href;
        this.frameTarget.src = href;
    }
}