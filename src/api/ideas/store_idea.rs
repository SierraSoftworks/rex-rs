use actix_web::{put, web};
use super::{AuthToken, APIError, ensure_user_collection};
use crate::models::*;
use super::{models, IdFilter, CollectionIdFilter};

#[put("/api/v1/idea/{id}")]
async fn store_idea_v1(
    (info, new_idea, state, token): (
        web::Path<IdFilter>,
        web::Json<models::IdeaV1>,
        web::Data<GlobalState>,
        AuthToken,
    ),
) -> Result<models::IdeaV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let idea: Idea = new_idea.into_inner().into();
    let id = parse_uuid!(info.id, idea ID);
    let uid = parse_uuid!(token.oid, auth token oid);

    ensure_user_collection(&state, &token).await?;

    state.store.send(StoreIdea {
        id: id,
        collection: uid,
        name: idea.name,
        description: idea.description,
        tags: idea.tags,
        completed: idea.completed,
    }).await?.map(|idea| idea.clone().into())
}

#[put("/api/v2/idea/{id}")]
async fn store_idea_v2(
    (info, new_idea, state, token): (
        web::Path<IdFilter>,
        web::Json<models::IdeaV2>,
        web::Data<GlobalState>,
        AuthToken,
    ),
) -> Result<models::IdeaV2, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let idea: Idea = new_idea.into_inner().into();
    let id = parse_uuid!(info.id, idea ID);
    let uid = parse_uuid!(token.oid, auth token oid);

    ensure_user_collection(&state, &token).await?;

    state.store.send(StoreIdea {
        id: id,
        collection: uid,
        name: idea.name,
        description: idea.description,
        tags: idea.tags,
        completed: idea.completed,
    }).await?.map(|idea| idea.clone().into())
}

#[put("/api/v3/idea/{id}")]
async fn store_idea_v3(
    (info, new_idea, state, token): (
        web::Path<IdFilter>,
        web::Json<models::IdeaV3>,
        web::Data<GlobalState>,
        AuthToken,
    ),
) -> Result<models::IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let idea: Idea = new_idea.into_inner().into();
    let id = parse_uuid!(info.id, idea ID);
    let uid = parse_uuid!(token.oid, auth token oid);

    ensure_user_collection(&state, &token).await?;

    state.store.send(StoreIdea {
        id: id,
        collection: uid,
        name: idea.name,
        description: idea.description,
        tags: idea.tags,
        completed: idea.completed,
    }).await?.map(|idea| idea.clone().into())
}

#[put("/api/v3/collection/{collection}/idea/{id}")]
async fn store_collection_idea_v3(
    (info, new_idea, state, token): (
        web::Path<CollectionIdFilter>,
        web::Json<models::IdeaV3>,
        web::Data<GlobalState>,
        AuthToken,
    ),
) -> Result<models::IdeaV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let idea: Idea = new_idea.into_inner().into();
    let id = parse_uuid!(info.id, idea ID);
    let cid = parse_uuid!(info.collection, collection ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    ensure_user_collection(&state, &token).await?;

    let role = state.store.send(GetRoleAssignment { principal_id: uid, collection_id: cid }).await??;

    match role.role {
        Role::Owner | Role::Contributor => {
            state.store.send(StoreIdea {
                id: id,
                collection: cid,
                name: idea.name,
                description: idea.description,
                tags: idea.tags,
                completed: idea.completed,
            }).await?.map(|idea| idea.clone().into())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to modify an idea within this collection."))
    }
}

#[cfg(test)]
mod tests {
    use crate::api::ideas::models::*;
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn store_idea_v1() {
        test_log_init();

        let content: IdeaV1 = test_request!(PUT "/api/v1/idea/00000000000000000000000000000001", IdeaV1 {
            id: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
        } => OK with content);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
    }

    #[actix_rt::test]
    async fn store_idea_v2() {
        test_log_init();

        let content: IdeaV2 = test_request!(PUT "/api/v2/idea/00000000000000000000000000000001", IdeaV2 {
            id: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => OK with content);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn store_idea_v3() {
        test_log_init();

        let content: IdeaV3 = test_request!(PUT "/api/v3/idea/00000000000000000000000000000001", IdeaV3 {
            id: None,
            collection: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => OK with content);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.collection, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }

    #[actix_rt::test]
    async fn store_collection_idea_v3() {
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

        let content: IdeaV3 = test_request!(PUT "/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001", IdeaV3 {
            id: None,
            collection: None,
            name: "Test Idea".to_string(),
            description: "This is a test idea".to_string(),
            tags: Some(hashset!("test")),
            completed: None
        } => OK with content | state = state);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.collection, Some("00000000000000000000000000000007".into()));
        assert_eq!(content.name, "Test Idea".to_string());
        assert_eq!(content.description, "This is a test idea".to_string());
        assert_eq!(content.tags, Some(hashset!("test")));
        assert_eq!(content.completed, Some(false));
    }
}