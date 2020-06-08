use actix_web::{get, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::{models, CollectionFilter};

#[get("/api/v3/collection/{collection}/users")]
async fn get_role_assignments_v3(
    (state, info, token): (web::Data<GlobalState>, web::Path<CollectionFilter>, AuthToken),
) -> Result<web::Json<Vec<models::RoleAssignmentV3>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "RoleAssignments.Write");
    
    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid, auth token oid);

    let role = state.store.send(GetRoleAssignment { collection_id: cid, principal_id: uid }).await??;
    match role.role {
        Role::Owner => state.store.send(GetRoleAssignments { collection_id: cid }).await?.map(|roles| web::Json(roles.iter().map(|i| i.clone().into()).collect())),
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
    async fn get_role_assignments_v3() {
        test_log_init();

        let state = GlobalState::new();

        state.store.send(StoreRoleAssignment {
            collection_id: 1,
            principal_id: 0,
            role: Role::Owner,
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        state.store.send(StoreRoleAssignment {
            collection_id: 1,
            principal_id: 2,
            role: Role::Viewer,
            ..Default::default()
        }).await.expect("the actor should run").expect("the idea should be stored");

        let mut app = get_test_app(state.clone()).await;

        let req = test::TestRequest::with_uri("/api/v3/collection/00000000000000000000000000000001/users")
            .method(Method::GET)
            .header("Authorization", auth_token()).to_request();

        let mut response = test::call_service(&mut app, req).await;
        assert_status(&mut response, StatusCode::OK).await;
        
        let content: Vec<RoleAssignmentV3> = get_content(&mut response).await;
        assert_eq!(content.len(), 2);

        for role in content {
            assert_eq!(role.collection_id, Some("00000000000000000000000000000001".into()));
            match role.user_id.unwrap().as_str() {
                "00000000000000000000000000000000" => {
                    assert_eq!(role.role, "Owner".to_string());
                },
                "00000000000000000000000000000002" => {
                    assert_eq!(role.role, "Viewer".to_string());
                },
                _ => assert!(false)
            }
        }
    }
}