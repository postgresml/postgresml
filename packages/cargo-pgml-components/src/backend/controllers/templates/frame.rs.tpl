use pgml_components::{Response, Error};
use rocket::form::Form;
use sailfish::TemplateOnce;

use <%= component.rust_path() %>;
<% if component.is_form() { %>use <%= component.rust_module() %>::forms;<% } %>

#[get("<%= component.frame_url() %>")]
pub async fn get() -> Result<Response, Error> {
    Ok(Response::ok(<%= component.rust_name() %>::new().render_once().unwrap()))
}
<% if component.is_form() { %>
#[post("<%= component.frame_url() %>/create", data = "<form>")]
pub async fn create(form: Form<forms::<%= component.rust_name() %>>) -> Result<Response, Error> {
    Ok(Response::redirect(format!("<%= component.frame_url() %>")))
}
<% } %>
