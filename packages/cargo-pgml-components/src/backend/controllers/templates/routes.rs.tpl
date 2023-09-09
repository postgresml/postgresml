use rocket::route::Route;

pub fn routes() -> Vec<Route> {
    routes![<% for component in components {
        if component.is_frame() { %>
        <%= component.rust_module() %>::frame::get,
        <%= component.rust_module() %>::frame::post,
        <%= component.rust_module() %>::frame::post_ok,<% } } %>
    ]
}
