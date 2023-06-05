import {
    Controller
} from '@hotwired/stimulus'

import { 
    createToast, 
    showToast 
} from './utilities/toast.js';

export default class extends Controller {
    codeCopy() {
        let text = [...this.element.querySelectorAll('span.code-content')]
            .map((copied) => copied.innerText)
            .join('\n')

        if (text.length === 0) {
            text = this.element.innerText.replace('content_copy', '')
        }

        text = text.trim()

        navigator.clipboard.writeText(text)

        const toastElement = createToast('Copied to clipboard');
        showToast(toastElement);
    }

}
