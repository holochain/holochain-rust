use holochain_core_types::{cas::content::Address, validation::ValidationPackage};

/// These are the different kinds of (low-level, i.e. non-app)
/// node-to-node messages that can be send between Holochain nodes.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum DirectMessage {
    /// A custom direct message is something that gets triggered
    /// from zome code, i.e. from the app.
    /// Receiving such a messages triggers a WASM callback
    Custom(String),

    /// This message is used to ask another node (which needs to
    /// be the author) for the validation package of a given entry.
    RequestValidationPackage(Address),

    /// With this message an author is responding to a
    /// RequestValidationPackage message.
    /// Option<> since there has to be a way to respond saying
    /// "I can't"
    ValidationPackage(Option<ValidationPackage>),
}
