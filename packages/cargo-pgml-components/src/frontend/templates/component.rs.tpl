use sailfish::TemplateOnce;
use pgml_components::component;
<% if component.is_frame() { %>
pub mod frame;
<% } %><% if component.is_form() { %>
pub mod forms; <% } %>

#[derive(TemplateOnce, Default)]
#[template(path = "<%= component.path() %>/template.html")]
pub struct <%= component.rust_name() %> {
    value: String,
}

impl <%= component.rust_name() %> {
    pub fn new() -> <%= component.rust_name() %> {
        <%= component.rust_name() %> {
            value: String::from("<%= component.full_path() %>"),
        }
    }
}

component!(<%= component.rust_name() %>);
