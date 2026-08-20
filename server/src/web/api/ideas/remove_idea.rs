use actix_web::{HttpResponse, http::StatusCode, web};
use rex_api::ApiError;
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::parse_id_or_400,
    services::Services,
    web::api::{CollectionIdFilter, IdFilter, ensure_user_collection},
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn remove_idea_v1<S: Services>(
    info: web::Path<IdFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<HttpResponse, ApiError> {
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let uid = token.principal_id();

    services.store().remove_idea(uid, id).await?;

    Ok(HttpResponse::build(StatusCode::NO_CONTENT).finish())
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn remove_idea_v2<S: Services>(
    info: web::Path<IdFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<HttpResponse, ApiError> {
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let uid = token.principal_id();

    services.store().remove_idea(uid, id).await?;

    Ok(HttpResponse::build(StatusCode::NO_CONTENT).finish())
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn remove_idea_v3<S: Services>(
    info: web::Path<IdFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<HttpResponse, ApiError> {
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let uid = token.principal_id();

    services.store().remove_idea(uid, id).await?;

    Ok(HttpResponse::build(StatusCode::NO_CONTENT).finish())
}

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn remove_collection_idea_v3<S: Services>(
    info: web::Path<CollectionIdFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<HttpResponse, ApiError> {
    let id = parse_id_or_400(&info.id, "idea ID")?;
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let uid = token.principal_id();

    ensure_user_collection(services.get_ref(), &token).await?;

    let role = services.store().get_role_assignment(cid, uid).await?;

    if !role.role.can_write() {
        return Err(ApiError::forbidden(
            "You do not have permission to remove an idea from this collection.",
        ));
    }

    services.store().remove_idea(cid, id).await?;

    Ok(HttpResponse::build(StatusCode::NO_CONTENT).finish())
}

#[cfg(test)]
mod tests {
    use crate::{db::Store, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn remove_idea_v1() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 0,
                    user_id: 0,
                    name: "My Ideas".into()
                },
                idea {
                    id: 1,
                    collection_id: 0,
                    ..Default::default()
                },
            ]
        );

        test_request!(DELETE "/api/v1/idea/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state
            .store()
            .get_idea(0, 1)
            .await
            .expect_err("The idea should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_idea_v2() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 0,
                    user_id: 0,
                    name: "My Ideas".into()
                },
                idea {
                    id: 1,
                    collection_id: 0,
                    ..Default::default()
                },
            ]
        );

        test_request!(DELETE "/api/v2/idea/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state
            .store()
            .get_idea(0, 1)
            .await
            .expect_err("The idea should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_idea_v3() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 0,
                    user_id: 0,
                    name: "My Ideas".into()
                },
                idea {
                    id: 1,
                    collection_id: 0,
                    ..Default::default()
                },
            ]
        );

        test_request!(DELETE "/api/v3/idea/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state
            .store()
            .get_idea(0, 1)
            .await
            .expect_err("The idea should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_collection_idea_v3() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 7,
                    user_id: 0,
                    name: "Test Collection".into()
                },
                idea {
                    id: 1,
                    collection_id: 7,
                    ..Default::default()
                },
            ]
        );

        test_request!(DELETE "/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state
            .store()
            .get_idea(7, 1)
            .await
            .expect_err("The idea should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_collection_idea_v3_as_a_viewer() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 7,
                    user_id: 9,
                    name: "Somebody Else's".into()
                },
                role {
                    collection_id: 7,
                    user_id: 0,
                    role: crate::models::Role::Viewer
                },
                idea {
                    id: 1,
                    collection_id: 7,
                    ..Default::default()
                },
            ]
        );

        test_request!(DELETE "/api/v3/collection/00000000000000000000000000000007/idea/00000000000000000000000000000001" => FORBIDDEN | state = state);

        state
            .store()
            .get_idea(7, 1)
            .await
            .expect("a viewer must not be able to delete an idea");
    }
}
