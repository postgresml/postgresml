import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = []
  static outlets = []

  initialize() {
    console.log('Initialized cards-newsletter-subscribe')
  }

  connect() {}

  disconnect() {}
}