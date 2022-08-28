use serde_json::json;

fn main() {
    println!("{}", serde_gron::to_string(&json!(1)).unwrap());
    println!("{}", serde_gron::to_string(&json!(true)).unwrap());
    println!("{}", serde_gron::to_string(&json!("abc")).unwrap());
    println!("{}", serde_gron::to_string(&json!([1, 2, 3])).unwrap());
    println!(
        "{}",
        serde_gron::to_string(&json!({"a": 1, "b": 2, "c": 3})).unwrap()
    );
}
