use serde::{Deserialize, Serialize};

use crate::schema::persons;


// This represents the payload of an HTTP Response
#[derive(Debug, Clone, Serialize, Queryable, Insertable)]
pub struct Person {
    pub id: i32,
    pub name: String,
}

// This represent the payload of an HTTP Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPerson {
    pub name: String,
}

// This represent the payload of an HTTP Request
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct UpdatePerson {
    pub name: String,
}
