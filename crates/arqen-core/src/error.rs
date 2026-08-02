use std::fmt;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    NotFound,
    Validation,
    Authentication,
    Authorization,
    Internal,
    External,
}

#[derive(Debug)]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl AppError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &dyn std::error::Error)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::NotFound => write!(f, "not found"),
            ErrorKind::Validation => write!(f, "validation error"),
            ErrorKind::Authentication => write!(f, "authentication error"),
            ErrorKind::Authorization => write!(f, "authorization error"),
            ErrorKind::Internal => write!(f, "internal error"),
            ErrorKind::External => write!(f, "external error"),
        }
    }
}
