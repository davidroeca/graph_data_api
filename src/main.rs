#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
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
fn get_node(id: ID, map: State<GraphMap>) -> JSON<Value> {
    let hash_map = map.lock().expect("Concurrency issue");
    match hash_map.get(&id) {
        Some(node) => JSON(json!({"output": Some(node)})),
        None => JSON(json!({"output": "No Data"}))
    }
}

#[post("/<id>", format = "application/json", data = "<node>")]
fn post_node(id: ID, node: JSON<Node>, map: State<GraphMap>) -> JSON<Value> {
    let mut hash_map = map.lock().expect("Concurrency issue");
    if hash_map.contains_key(&id) {
        JSON(json!({ "status": "Node already exists"}))
    } else {
        hash_map.insert(id, node.into_inner());
        JSON(json!({ "status": "ok" }))
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
