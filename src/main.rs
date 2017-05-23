#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;

#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket_contrib::{JSON, Value};
use rocket::State;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

type ID = usize;
type GraphMap = Mutex<HashMap<ID, Node>>;

#[derive(Serialize, Deserialize)]
struct Node {
    id: ID,
    parent: Option<ID>,
    children: Option<HashSet<ID>>,
}

impl Node {

    fn remove_parent(&mut self) {
        self.parent = None;
    }

    fn add_child(&mut self, id: ID) {
        if let Some(ref mut children) = self.children {
            children.insert(id);
        } else {
            let mut new_children = HashSet::new();
            new_children.insert(id);
            self.children = Some(new_children);
        }
    }

    fn remove_child(&mut self, id: ID) {
        if let Some(ref mut children) = self.children {
            children.remove(&id);
        }
    }
}

#[get("/<id>", format="application/json")]
fn get_node(id: ID, map: State<GraphMap>) -> JSON<Value> {
    let hash_map = map.lock().expect("Concurrency issue");
    if let Some(node) = hash_map.get(&id) {
        JSON(json!(node))
    } else {
        JSON(Value::Null)
    }
}

#[put("/<id>", format = "application/json")]
fn put_node(id: ID, map: State<GraphMap>) -> JSON<Value> {
    let mut hash_map = map.lock().expect("Concurrency issue");
    if hash_map.contains_key(&id) {
        JSON(json!({ "status": "Node already exists"}))
    } else {
        let node = Node {
            id: id,
            parent: None,
            children: None,
        };
        hash_map.insert(id, node);
        JSON(json!({ "status": "ok" }))
    }
}

#[put("/<id>/<child_id>", format="application/json")]
fn put_child(id: ID, child_id: ID, map: State<GraphMap>) -> JSON<Value> {
    let mut hash_map = map.lock().expect("Concurrency issue");
    if hash_map.contains_key(&child_id) {
        // Can't have child already existing
        return JSON(json!({
            "status": format!("child id {} exists", child_id)
        }));
    }
    if let Some(ref mut node) = hash_map.get_mut(&id) {
        // This branch required for explicit borrowing from hash_map
        node.add_child(child_id);
    } else {
        return JSON(json!({ "status": format!("id {} doesn't exist", id)}));
    }
    hash_map.insert(child_id, Node {
        id: child_id,
        parent: Some(id),
        children: None,
    });
    JSON(json!({ "status": "Child added"}))
}

#[delete("/<id>", format="application/json")]
fn delete_node(id: ID, map: State<GraphMap>) -> JSON<Value> {
    let mut hash_map = map.lock().expect("Concurrency issue");

    let opt_parent = match hash_map.get(&id) {
        Some(node) => node.parent,
        None => None,
    };

    match opt_parent {
        Some(parent_id) => {
            if let Some(parent) = hash_map.get_mut(&parent_id) {
                parent.remove_child(id);
            }
        },
        None => (),
    };

    let opt_children = match hash_map.get(&id) {
        Some(node) => node.children.clone(),
        None => None,
    };

    match opt_children {
        Some(children) => {
            for child_id in children.iter() {
                if let Some(child) = hash_map.get_mut(&child_id) {
                    child.remove_parent();
                }
            }
        },
        None => (),
    };

    hash_map.remove(&id);
    JSON(json!({ "status": "Node removed"}))
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![get_node, put_node, put_child, delete_node])
        .manage(Mutex::new(HashMap::<ID, Node>::new()))
}

fn main() {
    rocket().launch();
}
