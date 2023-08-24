use rust_bridge::{alias, alias_manual, alias_methods};

#[derive(alias, Debug, Clone)]
pub struct Test {
    x: i32,
}

#[alias_methods(new, get)]
impl Test {
    pub fn new(x: i32) -> Self {
        Self { x }
    }

    pub fn get(&self) -> i32{
        self.x
    }
}

fn main() {
    println!("Hello, world!");
}
