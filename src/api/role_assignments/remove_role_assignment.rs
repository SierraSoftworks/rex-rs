use actix_web::{delete, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::{CollectionUserFilter};

#[delete("/api/v3/collection/{collection}/user/{user}")]
async fn remove_role_assignment_v3(
    (info, state, token): (web::Path<CollectionUserFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::HttpResponse, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "RoleAssignments.Write");
    
    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid, auth token oid);
    let tuid = parse_uuid!(info.user, user ID);

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
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn remove_role_assignment_v3() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreRoleAssignment {
            collection_id: 1,
            principal_id: 0,
            role: Role::Owner,
        }).await.expect("the actor should run").expect("the role assignment should be stored");

        state.store.send(StoreRoleAssignment {
            collection_id: 1,
            principal_id: 2,
            role: Role::Viewer,
        }).await.expect("the actor should run").expect("the role assignment should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002")
            .method(Method::DELETE)
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::NO_CONTENT).await;

        state.store.send(GetRoleAssignment {
            collection_id: 1,
            principal_id: 2
        }).await.expect("the actor should have run").expect_err("The role assignment should not exist anymore");
    }
}