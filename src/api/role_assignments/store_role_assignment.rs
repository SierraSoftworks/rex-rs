use actix_web::{put, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::{models, CollectionUserFilter};

#[put("/api/v3/collection/{collection}/user/{user}")]
async fn store_role_assignment_v3(
    (info, collection, state, token): (web::Path<CollectionUserFilter>,
        web::Json<models::RoleAssignmentV3>,
        web::Data<GlobalState>, AuthToken),
) -> Result<models::RoleAssignmentV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "RoleAssignments.Write");
    
    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid, auth token oid);
    let tuid = parse_uuid!(info.user, user ID);
    
    let role = state.store.send(GetRoleAssignment { collection_id: cid, principal_id: uid }).await??;
    match role.role {
        Role::Owner => {
            state.store.send(StoreRoleAssignment {
                principal_id: tuid,
                collection_id: cid,
                role: collection.role.as_str().into(),
            }).await?.map(|collection| collection.clone().into())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to view or manage the list of users for this collection."))
    }
}

#[cfg(test)]
mod tests {
    use super::models::*;
    use crate::models::*;
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn store_role_assignment_v3() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreRoleAssignment {
            collection_id: 1,
            principal_id: 0,
            role: Role::Owner,
        }).await.expect("the actor should run").expect("the role assignment should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002")
            .method(Method::PUT)
            .set_json(&RoleAssignmentV3 {
                collection_id: None,
                user_id: None,
                role: "Owner".into(),
            })
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: RoleAssignmentV3 = get_content(&mut response).await;
        assert_eq!(content.collection_id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000002".into()));
        assert_eq!(content.role, "Owner".to_string());
    }
}