pub mod ticket;
pub mod user;
pub mod board;
pub mod sprint;

// Re-exports for convenience (will be used when UI is implemented)
#[allow(unused_imports)]
pub use ticket::Ticket;
#[allow(unused_imports)]
pub use user::User;
