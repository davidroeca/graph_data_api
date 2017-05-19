#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde;
extern crate serde_json;

#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket_contrib::{JSON, Value};
use rocket::State;
use std::collections::HashMap;
use std::sync::Mutex;

type ID = usize;
type GraphMap = Mutex<HashMap<ID, Node>>;

#[derive(Serialize, Deserialize)]
struct Node {
    id: ID,
    parent: Option<ID>,
    children: Option<Vec<ID>>,
}

#[get("/<id>", format="application/json")]
fn get_node(id: ID, map: State<GraphMap>) -> Option<JSON<&Node>> {
    let hash_map = map.lock().expect("Concurrency issue");
    hash_map.get(&id)
}

#[post("/<id>", format = "application/json", data = "<node>")]
fn post_node(id: ID, node: Node, map: State<GraphMap>) -> Option<JSON<Value>> {
    let mut hash_map = map.lock().expect("Concurrency issue");
    if hash_map.contains_key(&id) {
        None
    } else {
        hash_map.insert(id, node);
        Some(JSON(json!({ "status": "ok" })))
    }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![get_node, post_node])
        .manage(Mutex::new(HashMap::<ID, Node>::new()))
}

fn main() {
    rocket().launch();
}
