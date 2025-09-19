pub mod server;
pub mod routes;
pub mod handlers;
pub mod middleware;
pub mod error;
pub mod response;
pub mod state;
pub mod compat;

pub use server::ApiServer;
pub use error::{ApiError, ApiResult};
pub use response::ApiResponse;
pub use state::AppState;
pub use compat::StateWrapper;