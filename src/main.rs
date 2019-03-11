#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate serde;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use rocket::request::Form;
use rocket::request::Request;
use rocket::response::Redirect;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use std::env;

mod aaa;
mod model;
mod schema;

use model::Document;

// Hooks up the DB connection pool to Rocket
#[database("yaps")]
struct DB(PgConnection);

// The data model for submitted documents
// must be separate from model::Document because there's no way to ignore fields
// like `id` in an easy way.
#[derive(FromForm)]
struct DocumentSubmission {
    pub content: String,
}

// -
// Request handlers
// -

#[get("/")]
fn new() -> Template {
    Template::render("new", ())
}

#[get("/doc/<identifier>")]
fn doc<'a>(conn: DB, identifier: String) -> Result<Template, std::io::Error> {
    let document = match get_document(&conn, identifier) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            return Ok(Template::render("document", ()));
        }
    };
    println!("{:?}", document);
    Ok(Template::render("document", document))
}

#[post("/", data = "<document>")]
fn api_doc(
    conn: DB,
    document: Form<DocumentSubmission>,
) -> Result<Redirect, diesel::result::Error> {
    let identifier = crate::aaa::generate();
    create_document(
        &conn,
        Document {
            id: identifier.clone(),
            content: document.content.clone(),
            lang: String::from("markdown"),
        },
    )?;
    Ok(Redirect::to(format!("/doc/{}", &identifier)))
}

#[catch(404)]
fn not_found(_: &Request) -> Template {
    return Template::render("404", ());
}

// -
// Database stuff
// -

fn create_document(conn: &DB, doc: Document) -> Result<(), diesel::result::Error> {
    use schema::documents;

    diesel::insert_into(documents::table)
        .values(&doc)
        .execute(conn as &PgConnection)?;
    Ok(())
}

fn get_document(conn: &DB, identifier: String) -> Result<Document, diesel::result::Error> {
    use schema::documents::dsl::*;

    println!("searching for {}", identifier);

    documents
        .filter(id.eq(identifier))
        .first::<Document>(conn as &PgConnection)
}

fn db_connect() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

// -
// Entrypoint
// -

fn main() {
    dotenv().ok();

    rocket::ignite()
        .mount("/", routes![new, doc, api_doc])
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .register(catchers![not_found])
        .attach(DB::fairing())
        .attach(Template::fairing())
        .launch();
}
