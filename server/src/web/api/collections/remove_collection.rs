use actix_web::{HttpResponse, http::StatusCode, web};
use rex_api::ApiError;
use tracing::instrument;

use crate::{
    auth::AuthToken, db::Store, models::parse_id_or_400, services::Services,
    web::api::CollectionFilter,
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn remove_collection_v3<S: Services>(
    info: web::Path<CollectionFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<HttpResponse, ApiError> {
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let uid = token.principal_id();

    // An owner deletes the collection outright; anybody else is leaving it. The
    // store removes the caller's role assignment either way, so there is no
    // second call to make here.
    services.store().remove_collection(cid, uid).await?;

    Ok(HttpResponse::build(StatusCode::NO_CONTENT).finish())
}

#[cfg(test)]
mod tests {
    use crate::{db::Store, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn remove_collection_v3() {
        test_log_init();

        test_state!(
            state = [collection {
                collection_id: 1,
                user_id: 0,
                name: "Test Collection".into()
            },]
        );

        test_request!(DELETE "/api/v3/collection/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state
            .store()
            .get_collection(1, 0)
            .await
            .expect_err("The collection should not exist anymore");

        state
            .store()
            .get_role_assignment(1, 0)
            .await
            .expect_err("The role assignment should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_collection_v3_as_a_member_only_leaves_it() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 1,
                    user_id: 9,
                    name: "Shared Collection".into()
                },
                role {
                    collection_id: 1,
                    user_id: 0,
                    role: crate::models::Role::Viewer
                },
            ]
        );

        test_request!(DELETE "/api/v3/collection/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state
            .store()
            .get_collection(1, 0)
            .await
            .expect_err("the caller should have left the collection");

        state
            .store()
            .get_collection(1, 9)
            .await
            .expect("the owner should still have their collection");
    }
}
