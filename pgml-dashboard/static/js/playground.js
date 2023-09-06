import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  selectRow(event) {
    alert('Selected a row')
  }
}
