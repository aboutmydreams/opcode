use crate::commands::agents::AgentDb;
use crate::checkpoint::state::CheckpointState;
use crate::process::ProcessRegistryState;
use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub agent_db: Arc<AgentDb>,
    pub checkpoint_state: Arc<CheckpointState>,
    pub process_registry: Arc<ProcessRegistryState>,
}

impl AppState {
    pub fn new(
        agent_db: AgentDb,
        checkpoint_state: CheckpointState,
        process_registry: ProcessRegistryState,
    ) -> Self {
        Self {
            agent_db: Arc::new(agent_db),
            checkpoint_state: Arc::new(checkpoint_state),
            process_registry: Arc::new(process_registry),
        }
    }
}

// Enable extraction of individual components from AppState
impl FromRef<AppState> for Arc<AgentDb> {
    fn from_ref(state: &AppState) -> Self {
        state.agent_db.clone()
    }
}

impl FromRef<AppState> for Arc<CheckpointState> {
    fn from_ref(state: &AppState) -> Self {
        state.checkpoint_state.clone()
    }
}

impl FromRef<AppState> for Arc<ProcessRegistryState> {
    fn from_ref(state: &AppState) -> Self {
        state.process_registry.clone()
    }
}