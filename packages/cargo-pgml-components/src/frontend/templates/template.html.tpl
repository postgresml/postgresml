<% if component.is_frame() { %>
<turbo-frame src="<%= component.frame_url() %>" loading="lazy" id="<%= component.controller_name() %>">
<% } %>
<div data-controller="<%= component.controller_name() %>">
  <h3 class="text-center h3">
    <%%= value %>
  </h3>
  <% if component.is_form() { %>
  <form action="<%= component.frame_url() %>/create" method="post">
    <div class="mb-3">
      <label class="form-label">Text input</label>
      <input type="text" class="form-control" name="text_input" required>
    </div>
    <div class="mb-3">
      <label class="form-label">Number input</label>
      <input type="text" class="form-control" name="number_input" required>
    </div>
  </form>
  <% } %>
</div>
<% if component.is_frame() { %>
</turbo-frame>
<% } %>

