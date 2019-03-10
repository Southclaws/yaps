use serde::{Deserialize, Serialize};

use crate::schema::documents;

#[derive(FromForm, Deserialize, Serialize, Debug, Queryable, Insertable)]
pub struct Document {
    #[serde(default)]
    pub id: String,

    pub content: String,

    #[serde(default)]
    pub lang: String,
}
