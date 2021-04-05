use actix_web::{get, web};
use tracing::instrument;
use super::{AuthToken, APIError};
use crate::{models::*, telemetry::TraceMessageExt};
use super::CollectionUserFilter;

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v3/collection/{collection}/user/{user}")]
async fn get_role_assignment_v3(
    (info, state, token): (web::Path<CollectionUserFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<RoleAssignmentV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "RoleAssignments.Write");
    
    let cid = parse_uuid!(info.collection, "collection ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");
    let tuid = parse_uuid!(info.user, "user ID");

    if uid != tuid {
        let role = state.store.send(GetRoleAssignment { collection_id: cid, principal_id: uid }.trace()).await??;

        if role.role != Role::Owner {
            return Err(APIError::new(403, "Forbidden", "You do not have permission to view or manage the list of users for this collection."));
        }
    }

    state.store.send(GetRoleAssignment { collection_id: cid, principal_id: tuid }.trace()).await?.map(|role| role.clone().into())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_role_assignment_v3() {
        test_log_init();

        test_state!(state = [
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
        ]);

        let content: RoleAssignmentV3 = test_request!(GET "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002" => OK with content | state = state);
        assert_eq!(content.collection_id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000002".into()));
        assert_eq!(content.role, "Viewer".to_string());
    }

    #[actix_rt::test]
    async fn get_role_assignment_v3_self() {
        test_log_init();

        test_state!(state = [
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
        ]);

        let content: RoleAssignmentV3 = test_request!(GET "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000000" => OK with content | state = state);
        assert_eq!(content.collection_id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.role, "Owner".to_string());
    }
}