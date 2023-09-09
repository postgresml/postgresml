
#[derive(FromForm)]
pub struct <%= component.rust_name() %> {
    text_input: String,
    number_input: i64,
}
