import {
    Controller
} from '@hotwired/stimulus';

export default class extends Controller {
    connect() {
        const tables = this.getTables();

        Array.from(tables).forEach((table) => {
            let wrapper = this.makeWrapper();
            this.wrapElement(table, wrapper);
        });
    }

    getTables() {
        return this.element.getElementsByTagName("table");
    }

    makeWrapper() {
        let wrapper = document.createElement('div');
        wrapper.classList.add("overflow-auto", "w-100");
        return wrapper;
    }

    wrapElement(element, wrapper) {
        element.parentNode.replaceChild(wrapper, element);
        wrapper.appendChild(element);
    }
}
