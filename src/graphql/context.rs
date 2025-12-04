use async_graphql::Error;
use crate::db::models::User;

/// GraphQL context containing the current user (if authenticated)
#[derive(Debug, Clone)]
pub struct GraphQLContext {
  pub current_user: Option<User>,
}

impl GraphQLContext {
  pub fn new(current_user: Option<User>) -> Self {
    Self { current_user }
  }

  /// Require an authenticated user, or return an error
  pub fn require_user(&self) -> Result<&User, Error> {
    self.current_user
      .as_ref()
      .ok_or_else(|| Error::new("Authentication required"))
  }
}
