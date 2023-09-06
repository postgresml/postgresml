import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  selectRow(event) {
   console.log('dataset: ', event.currentTarget.dataset)
  }
}
