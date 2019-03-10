#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;

use rocket::request::Form;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use serde::Serialize;

mod aaa;

#[derive(FromForm, Serialize, Debug)]
struct Document {
    content: String,
}

#[get("/")]
fn new() -> Template {
    Template::render("new", ())
}

#[get("/doc/<identifier>")]
fn doc(identifier: String) -> Result<Template, std::io::Error> {
    let document = Document {
        content: std::fs::read_to_string(identifier)?,
    };
    Ok(Template::render("document", document))
}

#[post("/", data = "<document>")]
fn api_doc(document: Form<Document>) -> Result<rocket::response::Redirect, std::io::Error> {
    let identifier = crate::aaa::generate();
    std::fs::write(&identifier, &document.content)?;
    Ok(rocket::response::Redirect::to(format!(
        "/doc/{}",
        &identifier
    )))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![new, doc, api_doc])
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .attach(Template::fairing())
        .launch();
}
