use actix_web::{HttpResponse, http::StatusCode, web};
use rex_api::ApiError;
use tracing::instrument;

use crate::{
    auth::AuthToken,
    db::Store,
    models::{Role, parse_id_or_400},
    services::Services,
    web::api::CollectionUserFilter,
};

#[instrument(err, skip(services, token), fields(otel.kind = "internal"))]
pub async fn remove_role_assignment_v3<S: Services>(
    info: web::Path<CollectionUserFilter>,
    services: web::Data<S>,
    token: AuthToken,
) -> Result<HttpResponse, ApiError> {
    let cid = parse_id_or_400(&info.collection, "collection ID")?;
    let tuid = parse_id_or_400(&info.user, "user ID")?;
    let uid = token.principal_id();

    if tuid == uid {
        return Err(ApiError::bad_request(
            "You cannot remove yourself from a collection. Please request that another collection owner performs this for you.",
        ));
    }

    let role = services.store().get_role_assignment(cid, uid).await?;

    if role.role != Role::Owner {
        return Err(ApiError::forbidden(
            "You do not have permission to view or manage the list of users for this collection.",
        ));
    }

    services.store().remove_role_assignment(cid, tuid).await?;

    Ok(HttpResponse::build(StatusCode::NO_CONTENT).finish())
}

#[cfg(test)]
mod tests {
    use crate::{db::Store, models::Role, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn remove_role_assignment_v3() {
        test_log_init();

        test_state!(
            state = [
                collection {
                    collection_id: 1,
                    user_id: 0,
                    name: "Test Collection".into()
                },
                role {
                    collection_id: 1,
                    user_id: 2,
                    role: Role::Viewer
                },
            ]
        );

        test_request!(DELETE "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000002" => NO_CONTENT | state = state);

        state
            .store()
            .get_role_assignment(1, 2)
            .await
            .expect_err("The role assignment should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_role_assignment_v3_self() {
        test_log_init();

        test_state!(
            state = [collection {
                collection_id: 1,
                user_id: 0,
                name: "Test Collection".into()
            },]
        );

        test_request!(DELETE "/api/v3/collection/00000000000000000000000000000001/user/00000000000000000000000000000000" => BAD_REQUEST | state = state);

        state
            .store()
            .get_role_assignment(1, 0)
            .await
            .expect("The role assignment should still exist");
    }
}
