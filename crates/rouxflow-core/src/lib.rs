pub mod app_state;
pub mod bluetooth_manager;
pub mod cube;
pub mod session;
pub mod storage;
pub mod timer_manager;

pub use app_state::AppState;
pub use bluetooth_manager::BluetoothManager;
pub use session::SessionManager;
pub use timer_manager::TimerManager;

pub fn greet(name: &str) -> String {
    format!("Hello, {}! This is RouxFlow Core logic speaking.", name)
}
