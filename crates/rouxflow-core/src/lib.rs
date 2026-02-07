pub mod cube;
pub mod cube_manager;
pub mod session;
pub mod storage;

pub use cube_manager::CubeManager;

pub fn greet(name: &str) -> String {
    format!("Hello, {}! This is RouxFlow Core logic speaking.", name)
}
