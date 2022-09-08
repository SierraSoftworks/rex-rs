use actix_web::{HttpResponse, delete, web};
use tracing::instrument;
use super::{AuthToken, APIError, ensure_user_collection};
use crate::{models::*, telemetry::TraceMessageExt};
use super::{IdFilter, CollectionIdFilter};

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[delete("/api/v1/idea/{id}")]
async fn remove_idea_v1(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<HttpResponse, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let id = parse_uuid!(info.id, "idea ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");

    state.store.send(RemoveIdea { collection: uid, id }.trace()).await??;
    
    Ok(HttpResponse::build(http::StatusCode::NO_CONTENT).finish())
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[delete("/api/v2/idea/{id}")]
async fn remove_idea_v2(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<HttpResponse, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let id = parse_uuid!(info.id, "idea ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");
        
    state.store.send(RemoveIdea { collection: uid, id }.trace()).await??;
    
    Ok(HttpResponse::build(http::StatusCode::NO_CONTENT).finish())
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[delete("/api/v3/idea/{id}")]
async fn remove_idea_v3(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<HttpResponse, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let id = parse_uuid!(info.id, "idea ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");
        
    state.store.send(RemoveIdea { collection: uid, id }.trace()).await??;
    
    Ok(HttpResponse::build(http::StatusCode::NO_CONTENT).finish())
}

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[delete("/api/v3/collection/{collection}/idea/{id}")]
async fn remove_collection_idea_v3(
    (info, state, token): (web::Path<CollectionIdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<HttpResponse, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Ideas.Write");
    
    let id = parse_uuid!(info.id, "idea ID");
    let cid = parse_uuid!(info.collection, "collection ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");
        
    ensure_user_collection(&state, &token).await?;

    let role = state.store.send(GetRoleAssignment { principal_id: uid, collection_id: cid }.trace()).await??;

    match role.role {
        Role::Owner | Role::Contributor => {
            state.store.send(RemoveIdea { collection: cid, id }.trace()).await??;
            
            Ok(HttpResponse::build(http::StatusCode::NO_CONTENT).finish())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to remove an idea from this collection."))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn remove_idea_v1() {
        test_log_init();

        test_state!(state = [
            StoreIdea {
                id: 1,
                collection: 0,
                ..Default::default()
            }
        ]);

        test_request!(DELETE "/api/v1/idea/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state.store.send(GetIdea {
            collection: 0,
            id: 1
        }).await.expect("the actor should have run").expect_err("The idea should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_idea_v2() {
        test_log_init();

        test_state!(state = [
            StoreIdea {
                id: 1,
                collection: 0,
                ..Default::default()
            }
        ]);

        test_request!(DELETE "/api/v2/idea/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state.store.send(GetIdea {
            collection: 0,
            id: 1
        }).await.expect("the actor should have run").expect_err("The idea should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_idea_v3() {
        test_log_init();

        test_state!(state = [
            StoreIdea {
                id: 1,
                collection: 0,
                ..Default::default()
            }
        ]);

        test_request!(DELETE "/api/v3/idea/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state.store.send(GetIdea {
            collection: 0,
            id: 1
        }).await.expect("the actor should have run").expect_err("The idea should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_collection_idea_v3() {
        test_log_init();

        test_state!(state = [
            StoreCollection {
                collection_id: 7,
                principal_id: 0,
                name: "Test Collection".into(),
            },
            StoreRoleAssignment {
                collection_id: 7,
                principal_id: 0,
                role: Role::Owner,
            },
            StoreIdea {
                id: 1,
                collection: 7,
                ..Default::default()
            }
        ]);

        test_request!(DELETE "/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state.store.send(GetIdea {
            collection: 0,
            id: 1
        }).await.expect("the actor should have run").expect_err("The idea should not exist anymore");
    }
}