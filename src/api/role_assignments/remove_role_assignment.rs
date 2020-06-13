use actix_web::{delete, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::CollectionUserFilter;

#[delete("/api/v3/collection/{collection}/user/{user}")]
async fn remove_role_assignment_v3(
    (info, state, token): (web::Path<CollectionUserFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::HttpResponse, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "RoleAssignments.Write");
    
    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid, auth token oid);
    let tuid = parse_uuid!(info.user, user ID);

    if tuid == uid {
        return Err(APIError::new(400, "Bad Request", "You cannot remove yourself from a collection. Please request that another collection owner performs this for you."))
    }

    let role = state.store.send(GetRoleAssignment { collection_id: cid, principal_id: uid }).await??;
    match role.role {
        Role::Owner => {
            state.store.send(RemoveRoleAssignment { collection_id: cid, principal_id: tuid }).await??;

            Ok(web::HttpResponse::NoContent().finish())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to view or manage the list of users for this collection."))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn remove_role_assignment_v3() {
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

        test_request!(DELETE "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002" => NO_CONTENT | state = state);

        state.store.send(GetRoleAssignment {
            collection_id: 1,
            principal_id: 2
        }).await.expect("the actor should have run").expect_err("The role assignment should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_role_assignment_v3_self() {
        test_log_init();

        test_state!(state = [
            StoreRoleAssignment {
                collection_id: 1,
                principal_id: 0,
                role: Role::Owner,
            }
        ]);

        test_request!(DELETE "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000000" => BAD_REQUEST | state = state);

        state.store.send(GetRoleAssignment {
            collection_id: 1,
            principal_id: 0
        }).await.expect("the actor should have run").expect("The role assignment should still exist");
    }
}