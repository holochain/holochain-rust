pub enum WalkmanEvent {
    CoreAction(Action),
    // Lib3hClientProtocol,
}

impl From<Action> for WalkmanEvent {
    fn from(&self, action: Action) -> Self {
        CoreAction(action)
    }
}
