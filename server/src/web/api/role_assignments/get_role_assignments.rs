use actix_web::web;
use rex_api::{ApiError, RoleAssignmentV3};
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::{Role, parse_id_or_400},
    services::Services,
    web::api::CollectionFilter,
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn get_role_assignments_v3<S: Services>(
    services: web::Data<S>,
    info: web::Path<CollectionFilter>,
    token: AuthToken,
) -> Result<web::Json<Vec<RoleAssignmentV3>>, ApiError> {
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let uid = token.principal_id();

    let role = services.store().get_role_assignment(cid, uid).await?;

    if role.role != Role::Owner {
        return Err(ApiError::forbidden(
            "You do not have permission to view or manage the list of users for this collection.",
        ));
    }

    services
        .store()
        .get_role_assignments(cid)
        .await
        .map(|roles| web::Json(roles.into_iter().map(Into::into).collect()))
}

#[cfg(test)]
mod tests {
    use rex_api::RoleAssignmentV3;

    use crate::{models::Role, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn get_role_assignments_v3() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 1,
                    user_id: 0,
                    name: "Test Collection".into()
                },
                role {
                    collection_id: 1,
                    user_id: 2,
                    role: Role::Viewer
                },
            ]
        );

        let content: Vec<RoleAssignmentV3> = test_request!(GET "/api/v3/collection/00000000000000000000000000000001/users" => OK with content | state = state);
        assert_eq!(content.len(), 2);

        for role in content {
            assert_eq!(
                role.collection_id,
                Some("00000000000000000000000000000001".into())
            );
            match role.user_id.unwrap().as_str() {
                "00000000000000000000000000000000" => {
                    assert_eq!(role.role, "Owner".to_string());
                }
                "00000000000000000000000000000002" => {
                    assert_eq!(role.role, "Viewer".to_string());
                }
                other => unreachable!("unexpected user {other}"),
            }
        }
    }

    #[actix_rt::test]
    async fn get_role_assignments_v3_as_a_viewer() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 1,
                    user_id: 9,
                    name: "Somebody Else's".into()
                },
                role {
                    collection_id: 1,
                    user_id: 0,
                    role: Role::Viewer
                },
            ]
        );

        test_request!(GET "/api/v3/collection/00000000000000000000000000000001/users" => FORBIDDEN | state = state);
    }
}
