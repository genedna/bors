use crate::permissions::PermissionType;
use parking_lot::Mutex;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use wiremock::matchers::path;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

use crate::tests::mocks::repository::Repo;
use crate::tests::mocks::{User, World};
use crate::TeamApiClient;

#[derive(Clone, Default)]
pub struct Permissions {
    pub users: HashMap<User, Vec<PermissionType>>,
}

pub struct TeamApiMockServer {
    mock_server: MockServer,
}

impl TeamApiMockServer {
    pub async fn start(world: &World) -> Self {
        let mock_server = MockServer::start().await;

        let add_mock = |repo: Arc<Mutex<Repo>>, kind: PermissionType, name: &str| {
            let repo = repo.lock();
            let mut users = repo.permissions.users.iter().collect::<Vec<_>>();
            users.sort_by_key(|p| p.0.github_id);
            users.retain(|p| p.1.contains(&kind));
            let permissions = json!({
                "github_ids": users.into_iter().map(|(user, _)| user.github_id).collect::<Vec<_>>()
            });

            Mock::given(method("GET"))
                .and(path(format!(
                    "/v1/permissions/bors.{}.{name}.json",
                    repo.name.name()
                )))
                .respond_with(ResponseTemplate::new(200).set_body_json(permissions))
        };

        for repo in world.repos.values() {
            add_mock(repo.clone(), PermissionType::Review, "review")
                .mount(&mock_server)
                .await;
            add_mock(repo.clone(), PermissionType::Try, "try")
                .mount(&mock_server)
                .await;
        }

        Self { mock_server }
    }

    pub fn client(&self) -> TeamApiClient {
        TeamApiClient::new(self.mock_server.uri())
    }
}
