
import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {
    static targets = [
        'frame',
    ];

    connect() {
        let interval = 5000; // 5 seconds

        if (this.hasFrameTarget) {
            const frame = this.frameTarget.querySelector('turbo-frame')

            if (this.frameTarget.dataset.interval) {
                let value = parseInt(this.frameTarget.dataset.interval)
                if (!isNaN(value)) {
                    interval = value
                }
            }
        }

        if (this.hasFrameTarget) {
            const frame = this.frameTarget.querySelector('turbo-frame')

            if (frame) {
                this.interval = setInterval(() => {
                    const frame = this.frameTarget.querySelector('turbo-frame')
                    const src = `${frame.src}`
                    frame.src = null
                    frame.src = src
                }, interval);
            }
        }
    }

    disconnect() {
        clearTimeout(this.interval)
    }
}
