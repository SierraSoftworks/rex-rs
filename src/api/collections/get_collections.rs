use actix_web::{get, web};
use tracing::instrument;
use super::{AuthToken, APIError};
use crate::{api::ensure_user_collection, models::*, telemetry::TraceMessageExt};

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[get("/api/v3/collections")]
async fn get_collections_v3(
    (state, token): (web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<CollectionV3>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Collections.Read");

    let uid = parse_uuid!(token.oid(), "auth token oid");

    ensure_user_collection(&state, &token).await?;
        
    state.store.send(GetCollections { principal_id: uid }.trace()).await?.map(|ideas| web::Json(ideas.iter().map(|i| i.clone().into()).collect()))
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_collections_v3() {
        test_log_init();

        test_state!(state = [
            StoreCollection {
                collection_id: 1,
                principal_id: 0,
                name: "Test Collection".into(),
                ..Default::default()
            }
        ]);

        let content: Vec<CollectionV3> = test_request!(GET "/api/v3/collections" => OK with content | state = state);
        assert!(content.len() >= 1);
        assert!(content.iter().any(|c| c.id == Some("00000000000000000000000000000001".into())));
        assert!(content.iter().all(|c| c.user_id == Some("00000000000000000000000000000000".into())));
        assert!(content.iter().any(|c| c.name == "Test Collection"));
    }
}