use actix_web::{get, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::{models, CollectionUserFilter};

#[get("/api/v3/collection/{collection}/user/{user}")]
async fn get_role_assignment_v3(
    (info, state, token): (web::Path<CollectionUserFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<models::RoleAssignmentV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "RoleAssignments.Write");
    
    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid, auth token oid);
    let tuid = parse_uuid!(info.user, user ID);

    if uid != tuid {
        let role = state.store.send(GetRoleAssignment { collection_id: cid, principal_id: uid }).await??;

        if role.role != Role::Owner {
            return Err(APIError::new(403, "Forbidden", "You do not have permission to view or manage the list of users for this collection."));
        }
    }

    state.store.send(GetRoleAssignment { collection_id: cid, principal_id: tuid }).await?.map(|role| role.clone().into())
}

#[cfg(test)]
mod tests {
    use super::models::*;
    use crate::models::*;
    use actix_web::test;
    use http::{Method, StatusCode};
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_role_assignment_v3() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreRoleAssignment {
            collection_id: 1,
            principal_id: 0,
            role: Role::Owner,
        }).await.expect("the actor should run").expect("the idea should be stored");
        state.store.send(StoreRoleAssignment {
            collection_id: 1,
            principal_id: 2,
            role: Role::Viewer,
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002")
            .method(Method::GET)
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: RoleAssignmentV3 = get_content(&mut response).await;
        assert_eq!(content.collection_id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000002".into()));
        assert_eq!(content.role, "Viewer".to_string());
    }

    #[actix_rt::test]
    async fn get_role_assignment_v3_self() {
        test_log_init();

        let state = GlobalState::new();
        state.store.send(StoreRoleAssignment {
            collection_id: 1,
            principal_id: 0,
            role: Role::Owner,
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000000")
            .method(Method::GET)
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_eq!(response.status(), StatusCode::OK);
        
        let content: RoleAssignmentV3 = get_content(&mut response).await;
        assert_eq!(content.collection_id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.role, "Owner".to_string());
    }
}