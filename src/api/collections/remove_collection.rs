use super::CollectionFilter;
use super::{APIError, AuthToken};
use crate::{models::*, telemetry::TraceMessageExt};
use actix_web::{delete, web, HttpResponse};
use tracing::instrument;

#[instrument(err, skip(state, token), fields(otel.kind = "internal"))]
#[delete("/api/v3/collection/{collection}")]
async fn remove_collection_v3(
    (info, state, token): (
        web::Path<CollectionFilter>,
        web::Data<GlobalState>,
        AuthToken,
    ),
) -> Result<HttpResponse, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Collections.Write");

    let cid = parse_uuid!(info.collection, "collection ID");
    let uid = parse_uuid!(token.oid(), "auth token oid");

    state
        .store
        .send(
            RemoveCollection {
                id: cid,
                principal_id: uid,
            }
            .trace(),
        )
        .await??;

    state
        .store
        .send(
            RemoveRoleAssignment {
                collection_id: cid,
                principal_id: uid,
            }
            .trace(),
        )
        .await??;

    Ok(HttpResponse::build(http::StatusCode::NO_CONTENT).finish())
}

#[cfg(test)]
mod tests {
    use crate::api::test::*;
    use crate::models::*;

    #[actix_rt::test]
    async fn remove_collection_v3() {
        test_log_init();

        test_state!(
            state = [
                StoreCollection {
                    collection_id: 1,
                    principal_id: 0,
                    name: "Test Collection".into(),
                },
                StoreRoleAssignment {
                    collection_id: 1,
                    principal_id: 0,
                    role: Role::Owner,
                }
            ]
        );

        test_request!(DELETE "/api/v3/collection/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state
            .store
            .send(GetCollection {
                id: 1,
                principal_id: 1,
            })
            .await
            .expect("the actor should have run")
            .expect_err("The collection should not exist anymore");

        state
            .store
            .send(GetRoleAssignment {
                collection_id: 1,
                principal_id: 1,
            })
            .await
            .expect("the actor should have run")
            .expect_err("The role assignment should not exist anymore");
    }
}
