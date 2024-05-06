use serde::{Serialize};

#[derive(Serialize)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: String,
    pub username: String,
    pub password: String,
}



// the output to our `create_user` handler
#[derive(Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
}
