use diesel::pg::PgConnection;

use crate::config::Config;

pub type Repo = gotham_middleware_diesel::Repo<PgConnection>;

pub fn create_repo(config: &Config) -> Repo {
    let db_url = config.server.db_url.as_str();

    Repo::new(db_url)
}