use actix_web::{get, web};
use super::{AuthToken, APIError, ensure_user_collection};
use crate::models::*;
use super::{IdFilter, CollectionIdFilter};

#[get("/api/v1/idea/{id}")]
async fn get_idea_v1(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let id = parse_uuid!(info.id, idea ID);
    let uid = parse_uuid!(token.oid, auth token oid);
    
    state.store.send(GetIdea { collection: uid, id: id }).await?.map(|idea| idea.clone().into())
}

#[get("/api/v2/idea/{id}")]
async fn get_idea_v2(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV2, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let id = parse_uuid!(info.id, idea ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    state.store.send(GetIdea { collection: uid, id: id }).await?.map(|idea| idea.clone().into())
}

#[get("/api/v3/idea/{id}")]
async fn get_idea_v3(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let id = parse_uuid!(info.id, idea ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    ensure_user_collection(&state, &token).await?;
    
    state.store.send(GetIdea { collection: uid, id: id }).await?.map(|idea| idea.clone().into())
}

#[get("/api/v3/collection/{collection}/idea/{id}")]
async fn get_collection_idea_v3(
    (info, state, token): (web::Path<CollectionIdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Read");
    
    let id = parse_uuid!(info.id, idea ID);
    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    ensure_user_collection(&state, &token).await?;

    state.store.send(GetRoleAssignment { principal_id: uid, collection_id: cid }).await??;

    state.store.send(GetIdea { collection: cid, id: id }).await?.map(|idea| idea.clone().into())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_idea_v1() {
        test_log_init();

        test_state!(state = [
            StoreIdea {
                id: 1,
                collection: 0,
                name: "Test Idea".into(),
                description: "This is a test idea".into(),
                tags: hashset!("test"),
                ..Default::default()
            }
        ]);

        let content: IdeaV1 = test_request!(GET "/api/v1/idea/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
    }

    #[actix_rt::test]
    async fn get_idea_v2() {
        test_log_init();

        test_state!(state = [
            StoreIdea {
                id: 1,
                collection: 0,
                name: "Test Idea".into(),
                description: "This is a test idea".into(),
                tags: hashset!("test"),
                ..Default::default()
            }
        ]);

        let content: IdeaV2 = test_request!(GET "/api/v2/idea/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn get_idea_v3() {
        test_log_init();

        test_state!(state = [
            StoreIdea {
                id: 1,
                collection: 0,
                name: "Test Idea".into(),
                description: "This is a test idea".into(),
                tags: hashset!("test"),
                ..Default::default()
            }
        ]);

        let content: IdeaV3 = test_request!(GET "/api/v3/idea/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.collection, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn get_collection_idea_v3() {
        test_log_init();

        test_state!(state = [
            StoreCollection {
                collection_id: 7,
                principal_id: 0,
                name: "Test Collection".into(),
                ..Default::default()
            },
            StoreRoleAssignment {
                collection_id: 7,
                principal_id: 0,
                role: Role::Owner,
            },
            StoreIdea {
                id: 1,
                collection: 7,
                name: "Test Idea".into(),
                description: "This is a test idea".into(),
                tags: hashset!("test"),
                ..Default::default()
            }
        ]);

        let content: IdeaV3 = test_request!(GET "/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.collection, Some("00000000000000000000000000000007".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        let user = state.store.send(GetUser { email_hash: u128::from_str_radix("05c1de2f5c5e7933bee97a499e818c5e", 16).expect("a valid hash") })
            .await.expect("the actor to have run").expect("the user should exist");
        assert_eq!(user.first_name, "Testy");
        assert_eq!(user.principal_id, 0);
    }
}