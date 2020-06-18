use serde_derive::*;

/// Sets the GitHub user id in request - if not already present.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GitHubUserId {
    pub id: i32,
}

impl GitHubUserId {
    pub fn default() -> Self {
        GitHubUserId { id: -1 }
    }
}
