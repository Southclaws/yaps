#![feature(proc_macro_hygiene, decl_macro, never_type, type_alias_enum_variants)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_json;
extern crate dotenv;
extern crate serde;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use rocket::outcome::IntoOutcome;
use rocket::request::Form;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::request::Request;
use rocket::response::Redirect;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use std::env;

mod aaa;
mod model;
mod schema;

use model::Document;
use model::User;

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

#[derive(FromForm)]
struct LoginRequest {
    pub username: String,
    pub password: String,
}

struct UserID(i32);

impl<'a, 'r> FromRequest<'a, 'r> for UserID {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> Outcome<UserID, !> {
        request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| UserID(id))
            .or_forward(())
    }
}

#[derive(Clone)]
struct Context(serde_json::Map<String, serde_json::Value>);

impl Default for Context {
    fn default() -> Context {
        let c = serde_json::Map::new();

        Context(c)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for &'a Context {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> Outcome<&'a Context, !> {
        let ctx = request.local_cache(|| {
            let conn = &request.guard::<DB>().unwrap();

            request
                .cookies()
                .get_private("user_id")
                .and_then(|cookie| cookie.value().parse().ok())
                .and_then(|user_id: i32| {
                    let mut obj = serde_json::Map::new();
                    obj.insert(
                        String::from("route"),
                        json!({
                            "path": request.uri().path(),
                            "query": request.uri().query()
                        }),
                    );

                    Some((user_id, obj))
                })
                .map(|(user_id, mut obj)| {
                    use schema::users::dsl::*;
                    match users
                        .filter(id.eq(user_id))
                        .first::<User>(conn as &PgConnection)
                    {
                        Ok(v) => {
                            obj.insert(String::from("id"), serde_json::to_value(v.id).expect(""));
                            obj.insert(
                                String::from("name"),
                                serde_json::to_value(v.name).expect(""),
                            );
                            obj.insert(
                                String::from("admin"),
                                serde_json::to_value(v.admin).expect(""),
                            );
                        }
                        Err(_) => (),
                    };
                    Context(obj.clone())
                })
                .or(Some(Context::default()))
                .expect("")
        });

        Outcome::Success(ctx)
    }
}

// -
// Request handlers
// -

#[get("/")]
fn new() -> Template {
    Template::render("new", ())
}

#[get("/login")]
fn login(conn: DB, ctx: &Context) -> Template {
    Template::render("login_page", ctx.0.clone())
}

#[post("/login", data = "<login>")]
fn login_request(conn: DB, login: Form<LoginRequest>) -> Redirect {
    if login.username != "s" {
        return Redirect::to("/login");
    }
    if login.password != "p" {
        return Redirect::to("/login");
    }

    return Redirect::to("/");
}

#[get("/doc/<identifier>")]
fn doc<'a>(conn: DB, ctx: &Context, identifier: String) -> Result<Template, std::io::Error> {
    let document = match get_document(&conn, identifier) {
        Ok(v) => v,
        Err(_) => {
            return Ok(Template::render("404", ctx.0.clone()));
        }
    };
    let mut ctx = ctx.0.clone();
    ctx.insert(
        String::from("document"),
        serde_json::to_value(document).expect(""),
    );
    Ok(Template::render("document", ctx))
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
        .mount("/", routes![new, login, login_request, doc, api_doc])
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .register(catchers![not_found])
        .attach(DB::fairing())
        .attach(Template::fairing())
        .launch();
}
