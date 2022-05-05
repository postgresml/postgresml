import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
	static targets = [
		"targetOption"
	]

	initialize() {
		this.target = null
	}

	selectTarget(event) {
		event.preventDefault()

		this.target = event.currentTarget.getAttribute("data-target")

		if (event.currentTarget.classList.contains("selected")) {
			event.currentTarget.classList.remove("selected")
		} else {
			event.currentTarget.classList.add("selected")
		}
	}
}
