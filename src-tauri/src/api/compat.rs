// This is a simple wrapper to create a tauri::State-like interface for HTTP handlers
pub struct StateWrapper<T>(pub T);

impl<T> std::ops::Deref for StateWrapper<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<&T> for StateWrapper<&T> {
    fn from(value: &T) -> Self {
        StateWrapper(value)
    }
}