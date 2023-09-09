use pgml_components::{Response, Error};
use rocket::form::Form;
use sailfish::TemplateOnce;

use <%= component.rust_path() %>;

#[derive(FromForm)]
pub struct <%= component.rust_name() %>Form {
	text_input: String,
	number_input: i64,
}

#[get("<%= component.frame_url() %>")]
pub async fn get() -> Result<Response, Error> {
	Ok(Response::ok(<%= component.rust_name() %>::new().render_once().unwrap()))
}

#[post("<%= component.frame_url() %>", data = "<form>")]
pub async fn post(form: Form<<%= component.rust_name() %>Form>) -> Result<Response, Error> {
	Ok(Response::redirect(format!("<%= component.frame_url() %>/ok")))
}

#[get("<%= component.frame_url() %>/ok")]
pub async fn post_ok() -> Result<Response, Error> {
	Ok(Response::ok(<%= component.rust_name() %>::new().render_once().unwrap()))
}
