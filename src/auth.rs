//! Authentication

use secrecy::SecretString;

/// Supported forms of Authentication
pub enum Auth {
    /// No authentication
    None,
    /// Basic HTTP authentication
    Token { email: String, token: SecretString },
}

impl Default for Auth {
    fn default() -> Self {
        Auth::None
    }
}
