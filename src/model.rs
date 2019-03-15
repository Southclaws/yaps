use serde::{Deserialize, Serialize};

use crate::schema::documents;
use crate::schema::users;

#[derive(FromForm, Deserialize, Serialize, Debug, Queryable, Insertable)]
pub struct Document {
    #[serde(default)]
    pub id: String,

    pub content: String,

    #[serde(default)]
    pub lang: String,
}

#[derive(FromForm, Deserialize, Serialize, Debug, Queryable, Insertable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub admin: bool,
}
