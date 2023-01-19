// from https://github.com/afcapel/stimulus-autocomplete/blob/main/src/autocomplete.js
import { Controller } from "@hotwired/stimulus"

const optionSelector = "[role='option']:not([aria-disabled])"
const activeSelector = "[aria-selected='true']"

export default class Autocomplete extends Controller {
  static targets = ["input", "hidden", "results"]
  static classes = ["selected"]
  static values = {
    ready: Boolean,
    submitOnEnter: Boolean,
    url: String,
    minLength: Number,
    delay: { type: Number, default: 300 },
  }
  static uniqOptionId = 0

  connect() {
    this.close()

    if(!this.inputTarget.hasAttribute("autocomplete")) this.inputTarget.setAttribute("autocomplete", "off")
    this.inputTarget.setAttribute("spellcheck", "false")

    this.mouseDown = false

    this.onInputChange = debounce(this.onInputChange, this.delayValue)

    this.inputTarget.addEventListener("keydown", this.onKeydown)
    this.inputTarget.addEventListener("blur", this.onInputBlur)
    this.inputTarget.addEventListener("input", this.onInputChange)
    this.resultsTarget.addEventListener("mousedown", this.onResultsMouseDown)
    this.resultsTarget.addEventListener("click", this.onResultsClick)

    if (this.inputTarget.hasAttribute("autofocus")) {
      this.inputTarget.focus()
    }

    this.readyValue = true
  }

  disconnect() {
    if (this.hasInputTarget) {
      this.inputTarget.removeEventListener("keydown", this.onKeydown)
      this.inputTarget.removeEventListener("blur", this.onInputBlur)
      this.inputTarget.removeEventListener("input", this.onInputChange)
    }

    if (this.hasResultsTarget) {
      this.resultsTarget.removeEventListener("mousedown", this.onResultsMouseDown)
      this.resultsTarget.removeEventListener("click", this.onResultsClick)
    }
  }

  sibling(next) {
    const options = this.options
    const selected = this.selectedOption
    const index = options.indexOf(selected)
    const sibling = next ? options[index + 1] : options[index - 1]
    const def = next ? options[0] : options[options.length - 1]
    return sibling || def
  }

  select(target) {
    const previouslySelected = this.selectedOption
    if (previouslySelected) {
      previouslySelected.removeAttribute("aria-selected")
      previouslySelected.classList.remove(...this.selectedClassesOrDefault)
    }

    target.setAttribute("aria-selected", "true")
    target.classList.add(...this.selectedClassesOrDefault)
    this.inputTarget.setAttribute("aria-activedescendant", target.id)
    target.scrollIntoView({ behavior: "smooth", block: "nearest" })
  }

  onKeydown = (event) => {
    const handler = this[`on${event.key}Keydown`]
    if (handler) handler(event)
  }

  onEscapeKeydown = (event) => {
    if (!this.resultsShown) return

    this.hideAndRemoveOptions()
    event.stopPropagation()
    event.preventDefault()
  }

  onArrowDownKeydown = (event) => {
    const item = this.sibling(true)
    if (item) this.select(item)
    event.preventDefault()
  }

  onArrowUpKeydown = (event) => {
    const item = this.sibling(false)
    if (item) this.select(item)
    event.preventDefault()
  }

  onTabKeydown = (event) => {
    const selected = this.selectedOption
    if (selected) this.commit(selected)
  }

  onEnterKeydown = (event) => {
    const selected = this.selectedOption
    if (selected && this.resultsShown) {
      this.commit(selected)
      if (!this.hasSubmitOnEnterValue) {
        event.preventDefault()
      }
    }
  }

  onInputBlur = () => {
    if (this.mouseDown) return
    this.close()
  }

  commit(selected) {
    if (selected.getAttribute("aria-disabled") === "true") return

    if (selected instanceof HTMLAnchorElement) {
      selected.click()
      this.close()
      return
    }

    const textValue = selected.getAttribute("data-autocomplete-label") || selected.textContent.trim()
    const value = selected.getAttribute("data-autocomplete-value") || textValue
    this.inputTarget.value = textValue

    if (this.hasHiddenTarget) {
      this.hiddenTarget.value = value
      this.hiddenTarget.dispatchEvent(new Event("input"))
      this.hiddenTarget.dispatchEvent(new Event("change"))
    } else {
      this.inputTarget.value = value
    }

    this.inputTarget.focus()
    this.hideAndRemoveOptions()

    this.element.dispatchEvent(
      new CustomEvent("autocomplete.change", {
        bubbles: true,
        detail: { value: value, textValue: textValue, selected: selected }
      })
    )
  }

  clear() {
    this.inputTarget.value = ""
    if (this.hasHiddenTarget) this.hiddenTarget.value = ""
  }

  onResultsClick = (event) => {
    if (!(event.target instanceof Element)) return
    const selected = event.target.closest(optionSelector)
    if (selected) this.commit(selected)
  }

  onResultsMouseDown = () => {
    this.mouseDown = true
    this.resultsTarget.addEventListener("mouseup", () => {
      this.mouseDown = false
    }, { once: true })
  }

  onInputChange = () => {
    this.element.removeAttribute("value")
    if (this.hasHiddenTarget) this.hiddenTarget.value = ""

    const query = this.inputTarget.value.trim()
    if (query && query.length >= this.minLengthValue) {
      this.fetchResults(query)
    } else {
      this.hideAndRemoveOptions()
    }
  }

  identifyOptions() {
    const prefix = this.resultsTarget.id || "stimulus-autocomplete"
    const optionsWithoutId = this.resultsTarget.querySelectorAll(`${optionSelector}:not([id])`)
    optionsWithoutId.forEach(el => el.id = `${prefix}-option-${Autocomplete.uniqOptionId++}`)
  }

  hideAndRemoveOptions() {
    this.close()
    this.resultsTarget.innerHTML = null
  }

  fetchResults = async (query) => {
    if (!this.hasUrlValue) return

    const url = this.buildURL(query)
    try {
      this.element.dispatchEvent(new CustomEvent("loadstart"))
      const html = await this.doFetch(url)
      this.replaceResults(html)
      this.element.dispatchEvent(new CustomEvent("load"))
      this.element.dispatchEvent(new CustomEvent("loadend"))
    } catch(error) {
      this.element.dispatchEvent(new CustomEvent("error"))
      this.element.dispatchEvent(new CustomEvent("loadend"))
      throw error
    }
  }

  buildURL(query) {
    const url = new URL(this.urlValue, window.location.href)
    const params = new URLSearchParams(url.search.slice(1))
    params.append("q", query)
    url.search = params.toString()

    return url.toString()
  }

  doFetch = async (url) => {
    const response = await myFetch(url, this.optionsForFetch())
    const html = await response.text()
    return html
  }

  replaceResults(html) {
    this.resultsTarget.innerHTML = html
    this.identifyOptions()
    if (!!this.options) {
      this.open()
    } else {
      this.close()
    }
  }

  open() {
    if (this.resultsShown) return

    this.resultsShown = true
    this.element.setAttribute("aria-expanded", "true")
    this.element.dispatchEvent(
      new CustomEvent("toggle", {
        detail: { action: "open", inputTarget: this.inputTarget, resultsTarget: this.resultsTarget }
      })
    )
  }

  close() {
    if (!this.resultsShown) return

    this.resultsShown = false
    this.inputTarget.removeAttribute("aria-activedescendant")
    this.element.setAttribute("aria-expanded", "false")
    this.element.dispatchEvent(
      new CustomEvent("toggle", {
        detail: { action: "close", inputTarget: this.inputTarget, resultsTarget: this.resultsTarget }
      })
    )
  }

  get resultsShown() {
    return !this.resultsTarget.hidden
  }

  set resultsShown(value) {
    this.resultsTarget.hidden = !value
  }

  get options() {
    return Array.from(this.resultsTarget.querySelectorAll(optionSelector))
  }

  get selectedOption() {
    return this.resultsTarget.querySelector(activeSelector)
  }

  get selectedClassesOrDefault() {
    return this.hasSelectedClass ? this.selectedClasses : ["active"]
  }

  optionsForFetch() {
    return { headers: { "X-Requested-With": "XMLHttpRequest" } } // override if you need
  }
}

const debounce = (fn, delay = 10) => {
  let timeoutId = null

  return (...args) => {
    clearTimeout(timeoutId)
    timeoutId = setTimeout(fn, delay)
  }
}

export { Autocomplete }
