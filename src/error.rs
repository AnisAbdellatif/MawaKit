/// Everything that can go wrong in MawaKit.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum MawaKitError {
    /// The page doesn't contain a `confData = { … }` assignment.
    #[error("the page contains no mosque configuration (confData)")]
    ConfDataNotFound,
    /// The `confData` object never closes.
    #[error("the mosque configuration (confData) is cut off")]
    ConfDataUnterminated,
    /// The configuration isn't valid JSON or lacks a required field.
    #[error("invalid mosque configuration: {reason}")]
    InvalidConfiguration {
        /// What was wrong.
        reason: String,
    },
}
