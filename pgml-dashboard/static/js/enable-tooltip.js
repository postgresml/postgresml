import {
    Controller
} from '@hotwired/stimulus';

export default class extends Controller {
    connect() {
        const tooltipTriggerList = this.element.querySelectorAll('[data-bs-toggle="tooltip"]')
        const tooltipList = [...tooltipTriggerList].map(tooltipTriggerEl => new bootstrap.Tooltip(tooltipTriggerEl))
    }
}
