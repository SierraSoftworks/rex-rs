use actix_web::{post, web};
use tracing::instrument;
use super::{AuthToken, APIError};
use crate::{models::*, telemetry::TraceMessageExt};

#[instrument(err, skip(state, token), fields(otel.kind = "server"))]
#[post("/api/v3/collections")]
async fn new_collection_v3(
    (collection, state, token): (
        web::Json<CollectionV3>,
        web::Data<GlobalState>, AuthToken),
) -> Result<CollectionV3, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Collections.Write");
    
    let uid = parse_uuid!(token.oid(), auth token oid);
        
    let collection = state.store.send(StoreCollection {
        principal_id: uid,
        collection_id: new_id(),
        name: collection.name.clone(),
    }.trace()).await??;

    state.store.send(StoreRoleAssignment {
        principal_id: uid,
        collection_id: collection.collection_id,
        role: Role::Owner
    }.trace()).await??;
    
    Ok(collection.into())
}

#[cfg(test)]
mod tests {
    use crate::api::test::*;
    use crate::models::*;

    #[actix_rt::test]
    async fn new_collection_v3() {
        test_log_init();
        
        let content: CollectionV3 = test_request!(POST "/api/v3/collections", CollectionV3 {
            id: None,
            user_id: None,
            name: "Test Collection".into(),
        } => CREATED with content);

        assert_ne!(content.id, None);
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Collection".to_string());
    }
}