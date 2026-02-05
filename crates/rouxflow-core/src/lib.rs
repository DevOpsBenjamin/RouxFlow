pub mod cube;
pub mod session;
pub mod storage;

pub fn greet(name: &str) -> String {
    format!("Hello, {}! This is RouxFlow Core logic speaking.", name)
}
