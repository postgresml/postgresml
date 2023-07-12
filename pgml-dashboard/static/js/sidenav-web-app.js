import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {

    static targets = [
        'toggler'
    ]

    connect() {
        this.navStates = ['collapsing', 'collapsed', 'expanding', 'expanded']
        this.events = ['hide.bs.collapse', 'hidden.bs.collapse', 'show.bs.collapse', 'shown.bs.collapse']

        this.events.forEach(event => {
            this.togglerTarget.addEventListener(event, () => {
                this.getAllAffected().forEach(item => this.toggle(item))
            })
        })
    }

    getAllAffected() {
        return this.element.querySelectorAll('.side-menu-affect')
    }

    toggle(item) {
        for (const [index, state] of this.navStates.entries()) {
            if( item.classList.contains(state)) {
                this.changeClass(this.navStates[(index+1)%4], item)
                return
            }
        }
    }

    changeClass(eClass, item) {
        this.navStates.map(c => item.classList.remove(c))
        item.classList.add(eClass)
    }

}