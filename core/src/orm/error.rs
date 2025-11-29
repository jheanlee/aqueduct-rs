#[derive(Debug)]
pub enum DbError {
  NotFound,
  DbErr(sea_orm::DbErr)
}

impl std::fmt::Display for DbError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NotFound => write!(f, "resource not found"),
      Self::DbErr(error) => write!(f, "database error: {error}")
    }
  }
}

impl std::error::Error for DbError {}

impl From<sea_orm::DbErr> for DbError {
  fn from(error: sea_orm::DbErr) -> Self {
    Self::DbErr(error)
  }
}