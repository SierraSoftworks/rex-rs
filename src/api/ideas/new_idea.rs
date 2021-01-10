use actix_web::{post, web};
use super::{AuthToken, APIError, ensure_user_collection};
use crate::models::*;
use super::CollectionFilter;

#[post("/api/v1/ideas")]
async fn new_idea_v1(
    (new_idea, state, token): (web::Json<IdeaV1>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let idea: Idea = new_idea.into_inner().into();
    let uid = parse_uuid!(token.oid(), auth token oid);

    ensure_user_collection(&state, &token).await?;

    state.store.send(StoreIdea {
        id: new_id(),
        collection: uid,
        name: idea.name,
        description: idea.description,
        tags: idea.tags,
        completed: false,
    }).await?.map(|idea| idea.clone().into())
}

#[post("/api/v2/ideas")]
async fn new_idea_v2(
    (new_idea, state, token): (web::Json<IdeaV2>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV2, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let idea: Idea = new_idea.into_inner().into();
    let uid = parse_uuid!(token.oid(), auth token oid);

    ensure_user_collection(&state, &token).await?;

    state.store.send(StoreIdea {
        id: new_id(),
        collection: uid,
        name: idea.name,
        description: idea.description,
        tags: idea.tags,
        completed: false,
    }).await?.map(|idea| idea.clone().into())
}

#[post("/api/v3/ideas")]
async fn new_idea_v3(
    (new_idea, state, token): (web::Json<IdeaV3>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let idea: Idea = new_idea.into_inner().into();
    let uid = parse_uuid!(token.oid(), auth token oid);

    ensure_user_collection(&state, &token).await?;

    state.store.send(StoreIdea {
        id: new_id(),
        collection: uid,
        name: idea.name,
        description: idea.description,
        tags: idea.tags,
        completed: false,
    }).await?.map(|idea| idea.clone().into())
}

#[post("/api/v3/collection/{collection}/ideas")]
async fn new_collection_idea_v3(
    (new_idea, info, state, token): (web::Json<IdeaV3>, web::Path<CollectionFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let idea: Idea = new_idea.into_inner().into();
    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid(), auth token oid);
        
    if cid == uid {
        ensure_user_collection(&state, &token).await?;
    }

    let role = state.store.send(GetRoleAssignment { principal_id: uid, collection_id: cid }).await??;

    match role.role {
        Role::Owner | Role::Contributor => {
            state.store.send(StoreIdea {
                id: new_id(),
                collection: cid,
                name: idea.name,
                description: idea.description,
                tags: idea.tags,
                completed: idea.completed,
            }).await?.map(|idea| idea.clone().into())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to add an idea to this collection."))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn new_idea_v1() {
        test_log_init();

        test_state!(state = []);

        let content: IdeaV1 = test_request!(POST "/api/v1/ideas", IdeaV1 {
            id: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
        } => CREATED with location =~ "/api/v1/idea/", content | state = state);

        assert_ne!(content.id, None);
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());

        state.store.send(GetIdea {
            collection: 0,
            id: u128::from_str_radix(content.id.unwrap().as_str(), 16).unwrap(),
        }).await.expect("the actor should have run").expect("The idea should exist in the store");
    }

    #[actix_rt::test]
    async fn new_idea_v2() {
        test_log_init();

        test_state!(state = []);

        let content: IdeaV2 = test_request!(POST "/api/v2/ideas", IdeaV2 {
            id: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => CREATED with location =~ "/api/v2/idea/", content | state = state);

        assert_ne!(content.id, None);
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        state.store.send(GetIdea {
            collection: 0,
            id: u128::from_str_radix(content.id.unwrap().as_str(), 16).unwrap(),
        }).await.expect("the actor should have run").expect("The idea should exist in the store");
    }

    #[actix_rt::test]
    async fn new_idea_v3() {
        test_log_init();

        test_state!(state = []);

        let content: IdeaV3 = test_request!(POST "/api/v3/ideas", IdeaV3 {
            id: None,
            collection: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => CREATED with location =~ "/api/v3/idea/", content | state = state);

        assert_ne!(content.id, None);
        assert_eq!(content.collection, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        state.store.send(GetIdea {
            collection: 0,
            id: u128::from_str_radix(content.id.unwrap().as_str(), 16).unwrap(),
        }).await.expect("the actor should have run").expect("The idea should exist in the store");
    }

    #[actix_rt::test]
    async fn new_collection_idea_v3() {
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
            }
        ]);

        let content: IdeaV3 = test_request!(POST "/api/v3/collection/00000000000000000000000000000007/ideas", IdeaV3 {
            id: None,
            collection: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => CREATED with location =~ "/api/v3/collection/00000000000000000000000000000007/idea/", content | state = state);

        assert_ne!(content.id, None);
        assert_eq!(content.collection, Some("00000000000000000000000000000007".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));

        state.store.send(GetIdea {
            collection: 7,
            id: u128::from_str_radix(content.id.unwrap().as_str(), 16).unwrap(),
        }).await.expect("the actor should have run").expect("The idea should exist in the store");
    }
}