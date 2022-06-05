extern crate rust_project;
extern crate diesel;

use self::rust_project::*;
use self::rust_project::models::*;
use self::diesel::prelude::*;

fn main() {
    use rust_project::schema::messages::dsl::*;

    let connection = establish_connection();
    let results = messages
        .limit(5)
        .load::<Message>(&connection)
        .expect("Error loading posts");

    println!("Displaying {} messages", results.len());
    for message in results {
        println!("{}", message.sender);
        println!("----------\n");
        println!("{}", message.body);
    }
}