import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {

    connect() {
        let navbarMenues = document.querySelectorAll('.navbar-collapse');

        document.addEventListener('show.bs.collapse', e => {
            this.closeOtherMenues(navbarMenues, e.target)
        })

        document.addEventListener('hidden.bs.collapse', e => {
            this.closeSubmenus(e.target.querySelectorAll('.drawer-submenu'))
        })
    }

    closeOtherMenues(menus, current) {
        menus.forEach( menu => {
            const bsInstance = bootstrap.Collapse.getInstance(menu)
            if ( bsInstance && menu != current && menu != current.parentElement ) {
                bsInstance.hide()
            }
        })
    }

    closeSubmenus(submenues) {
        submenues.forEach(submenu => {
            const bsInstance = bootstrap.Collapse.getInstance(submenu)
            if ( bsInstance ) {
                bsInstance.hide()
            }
        })
    }
}
