import { Controller } from '@hotwired/stimulus'

export default class extends Controller {

  static targets = [
    'self'
  ];

  play(event) {
    event.preventDefault()
    const form = new FormData(event.currentTarget)
    const body = new URLSearchParams(form)

    fetch(event.currentTarget.action, {
      method: 'POST',
      body: body,
    })
    .then(res => res.text())
    .then(html => {
      const template = document.createElement('template')
      template.innerHTML = html.trim()
      this.selfTarget.parentNode.replaceChild(template.content.firstChild, this.selfTarget)
    })
  }
}
