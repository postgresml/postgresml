import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = []
  static outlets = []

  initialize() {
    console.log('Initialized notifications-marketing-feature-banner')
  }

  connect() {}

  disconnect() {}
}