pub mod context;
pub mod mutation;
pub mod query;
pub mod types;

use async_graphql::{EmptySubscription, Schema};
use mutation::MutationRoot;
use query::QueryRoot;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema() -> AppSchema {
  Schema::build(QueryRoot, MutationRoot, EmptySubscription)
    .finish()
}
