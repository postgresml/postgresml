import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {
    connect() {
        document.addEventListener('show.bs.collapse', this.closeOtherMenus)
        document.addEventListener('hidden.bs.collapse', this.closeSubmenus, false)
    }

    closeSubmenus(e) {
        let submenus = e.target.querySelectorAll('.drawer-submenu')
        submenus.forEach(submenu => {
            const bsInstance = bootstrap.Collapse.getInstance(submenu)
            if ( bsInstance ) {
                bsInstance.hide()
            }
        })
    }

    closeOtherMenus(e) {
        let menus = document.querySelectorAll('.navbar-collapse')
        let current = e.target

        menus.forEach( menu => {
            const bsInstance = bootstrap.Collapse.getInstance(menu)
            if ( bsInstance && menu != current && menu != current.parentElement ) {
                bsInstance.hide()
            }
        })
    }

    disconnect() {
        document.removeEventListener('show.bs.collapse', this.closeOtherMenus)
        document.removeEventListener('hidden.bs.collapse', this.closeSubmenus)
    }
}
