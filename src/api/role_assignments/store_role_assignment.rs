use actix_web::{put, web};
use tracing::instrument;
use super::{AuthToken, APIError};
use crate::{models::*, telemetry::TraceMessageExt};
use super::CollectionUserFilter;

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[put("/api/v3/collection/{collection}/user/{user}")]
async fn store_role_assignment_v3(
    (info, collection, state, token): (web::Path<CollectionUserFilter>,
        web::Json<RoleAssignmentV3>,
        web::Data<GlobalState>, AuthToken),
) -> Result<RoleAssignmentV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "RoleAssignments.Write");
    
    let cid = parse_uuid!(info.collection, "collection ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");
    let tuid = parse_uuid!(info.user, "user ID");
    
    let original_collection = state.store.send(GetCollection {
        id: cid,
        principal_id: uid
    }.trace()).await??;

    if tuid == uid {
        return Err(APIError::new(400, "Bad Request", "You cannot modify your own role assignment. Please request that another collection owner performs this task for you."))
    }

    let role = state.store.send(GetRoleAssignment { collection_id: cid, principal_id: uid }.trace()).await??;
    match role.role {
        Role::Owner => {
            match state.store.send(GetCollection {
                principal_id: tuid,
                id: cid
            }.trace()).await? {
                Ok(_) => {},
                Err(err) if err.code == 404 => {
                    state.store.send(StoreCollection {
                        principal_id: tuid,
                        collection_id: cid,
                        name: original_collection.name
                    }.trace()).await??;
                },
                Err(err) => {
                    return Err(err)
                }
            }

            state.store.send(StoreRoleAssignment {
                principal_id: tuid,
                collection_id: cid,
                role: collection.role.as_str().into(),
            }.trace()).await?.map(|collection| collection.clone().into())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to view or manage the list of users for this collection."))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn store_role_assignment_v3() {
        test_log_init();

        test_state!(state = [
            StoreCollection {
                collection_id: 1,
                principal_id: 0,
                name: "Test Collection".into()
            },
            StoreRoleAssignment {
                collection_id: 1,
                principal_id: 0,
                role: Role::Owner,
            }
        ]);

        let content: RoleAssignmentV3 = test_request!(PUT "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002", RoleAssignmentV3{
            collection_id: None,
            user_id: None,
            role: "Owner".into(),
        } => OK with content | state = state);

        assert_eq!(content.collection_id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000002".into()));
        assert_eq!(content.role, "Owner".to_string());

        let collection = state.store.send(GetCollection {
            id: 1,
            principal_id: 2
        }).await.expect("the actor should run").expect("the user should have the new collection");
        assert_eq!(collection.name, "Test Collection");
    }

    #[actix_rt::test]
    async fn store_role_assignment_v3_self() {
        test_log_init();

        test_state!(state = [
            StoreCollection {
                collection_id: 1,
                principal_id: 0,
                name: "Test Collection".into()
            },
            StoreRoleAssignment {
                collection_id: 1,
                principal_id: 0,
                role: Role::Owner,
            }
        ]);

        test_request!(PUT "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000000", RoleAssignmentV3{
            collection_id: None,
            user_id: None,
            role: "Viewer".into(),
        } => BAD_REQUEST | state = state);
    }
}