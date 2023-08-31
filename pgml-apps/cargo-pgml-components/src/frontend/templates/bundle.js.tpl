import { Application } from '@hotwired/stimulus'
const application = Application.start()

<% for component in components {
import { default as <%= component.name() %> } from '../../<%= component.controller_path() %>'"
application.register('<%= component.controller_name() %>')
<% } %>
