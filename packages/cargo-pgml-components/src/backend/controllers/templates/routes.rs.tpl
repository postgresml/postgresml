use rocket::route::Route;

pub fn routes() -> Vec<Route> {
    routes![<% for component in components {
        if component.is_frame() { %>
        <%= component.rust_module() %>::frame::get, <% if component.is_form() { %>
        <%= component.rust_module() %>::frame::create, <% } } } %>
    ]
}
