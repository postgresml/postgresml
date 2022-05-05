import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
	static targets = [
		"tableOption",
		"preview",
		"objectiveOption",
		"columnName",
		"targetOption",
	]

	initialize() {
		this.table = null
		this.objective = null
		this.target = null
	}

	selectTable(event) {
		event.preventDefault()

		this.table = event.currentTarget.getAttribute("data-table-name")

		// Unselet all existing tables.
		this.tableOptionTargets.forEach((element) => {
			element.classList.remove("selected")
		})

		// Select this table.
		event.currentTarget.classList.add("selected")

		fetch(`/snapshots/preview?table=${this.table}`)
			.then(response => response.text())
			.then(html => {
				this.previewTargets.forEach((preview) => {
					preview.innerHTML = html
				})
			})

		fetch(`/snapshots/targets?table=${this.table}`)
			.then(response => response.text())
			.then(html => {
				this.targetOptionTargets.forEach((target) => {
					target.innerHTML = html
				})
			})
	}

	selectObjective(event) {
		event.preventDefault()

		this.objective = event.currentTarget.getAttribute("data-objective")

		this.objectiveOptionTargets.forEach((element) => {
			element.classList.remove("selected")
		})

		event.currentTarget.classList.add("selected")
	}
}
