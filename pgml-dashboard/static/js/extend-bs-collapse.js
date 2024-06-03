// extends bootstraps collapse component by adding collapse state class to any 
// html element you like.  This is useful for adding style changes to elements 
// that do not need to collapse, when a collapse state change occurs. 
import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {

    static targets = [
        'stateReference'
    ]

    static values = {
        affected: String
    }

    connect() {
        this.navStates = ['collapsing', 'collapsed', 'expanding', 'expanded']
        this.events = ['hide.bs.collapse', 'hidden.bs.collapse', 'show.bs.collapse', 'shown.bs.collapse']

        this.callback = () =>  {
            this.getAllAffected().forEach(item => this.toggle(item))
        }

        this.events.forEach(event => {
            this.stateReferenceTarget.addEventListener(event, this.callback)
        })
    }

    getAllAffected() {
        return this.element.querySelectorAll(this.affectedValue)
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

    disconnect() {
        this.events.forEach(event => {
            this.stateReferenceTarget.removeEventListener(event, this.callback)
        })
    }
}
