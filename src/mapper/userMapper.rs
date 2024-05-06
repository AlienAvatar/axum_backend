extern crate mongodb;
use mongodb::bson::doc;
use mongodb::{Client, options::ClientOptions};
use crate::models::user::User;


pub async fn insert_user(client: &Client, db_name: &str, coll_name: &str, user:User) {
    let db: mongodb::Database = client.database(db_name);
    let coll = db.collection(coll_name);
    let doc = doc! { "name": user.username, "age": 30 };
    coll.insert_one(doc, None).await.unwrap();
    println!("Inserted a document into the collection!");
}