<% if component.is_frame() { %>
<turbo-frame src="<%= component.frame_url() %>" loading="lazy" id="<%= component.controller_name() %>">
<% } %>
<div data-controller="<%= component.controller_name() %>">
  <h3 class="text-center h3">
    <%%= value %>
  </h3>
</div>
<% if component.is_frame() { %>
</turbo-frame>
<% } %>

