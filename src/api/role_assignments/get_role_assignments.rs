use super::CollectionFilter;
use super::{APIError, AuthToken};
use crate::{models::*, telemetry::TraceMessageExt};
use actix_web::{get, web};
use tracing::instrument;

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v3/collection/{collection}/users")]
async fn get_role_assignments_v3(
    (state, info, token): (
        web::Data<GlobalState>,
        web::Path<CollectionFilter>,
        AuthToken,
    ),
) -> Result<web::Json<Vec<RoleAssignmentV3>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "RoleAssignments.Write");

    let cid = parse_uuid!(info.collection, "collection ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");

    let role = state
        .store
        .send(
            GetRoleAssignment {
                collection_id: cid,
                principal_id: uid,
            }
            .trace(),
        )
        .await??;
    match role.role {
        Role::Owner => state
            .store
            .send(GetRoleAssignments { collection_id: cid }.trace())
            .await?
            .map(|roles| web::Json(roles.iter().map(|i| i.clone().into()).collect())),
        _ => Err(APIError::new(
            403,
            "Forbidden",
            "You do not have permission to view or manage the list of users for this collection.",
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::api::test::*;
    use crate::models::*;

    #[actix_rt::test]
    async fn get_role_assignments_v3() {
        test_log_init();

        test_state!(
            state = [
                StoreRoleAssignment {
                    collection_id: 1,
                    principal_id: 0,
                    role: Role::Owner,
                },
                StoreRoleAssignment {
                    collection_id: 1,
                    principal_id: 2,
                    role: Role::Viewer,
                }
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
                _ => unreachable!(),
            }
        }
    }
}
