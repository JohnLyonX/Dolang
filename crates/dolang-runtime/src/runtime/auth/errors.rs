use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthConfigError {
    MissingJwtSecret,
    InvalidDefaultScheme(String),
    DefaultSchemeRequiresJwt,
    UnsupportedJwtAlgorithm(String),
}

impl fmt::Display for AuthConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingJwtSecret => write!(f, "JWT secret is required when JWT auth is enabled"),
            Self::InvalidDefaultScheme(scheme) => {
                write!(
                    f,
                    "auth default scheme must be 'session' or 'bearer', got '{scheme}'"
                )
            }
            Self::DefaultSchemeRequiresJwt => {
                write!(
                    f,
                    "auth default scheme 'bearer' requires [server.auth.jwt].enabled = true"
                )
            }
            Self::UnsupportedJwtAlgorithm(algorithm) => {
                write!(f, "unsupported JWT algorithm '{algorithm}'")
            }
        }
    }
}

impl std::error::Error for AuthConfigError {}
