pub fn run_hello(name: Option<String>) {
    let who = name.unwrap_or_else(|| "world".to_string());
    println!("Test command: Hello, {}!", who);
}
