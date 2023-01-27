use rocket::fs::TempFile;

#[derive(FromForm)]
pub struct Notebook<'a> {
    pub name: &'a str,
}

#[derive(FromForm)]
pub struct Cell<'a> {
    pub contents: &'a str,
    pub cell_type: &'a str,
}

#[derive(FromForm)]
pub struct Upload<'a> {
    pub file: TempFile<'a>,
    pub has_header: bool,
}
