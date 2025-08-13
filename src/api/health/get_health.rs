use crate::api::APIError;
use crate::{models::*, telemetry::TraceMessageExt};
use actix_web::{get, web};
use tracing::instrument;

#[instrument(err, skip(state), fields(otel.kind = "internal"))]
#[get("/api/v1/health")]
pub async fn get_health_v1(state: web::Data<GlobalState>) -> Result<HealthV1, APIError> {
    state
        .store
        .send(GetHealth {}.trace())
        .await?
        .map(|health| health.into())
}

#[instrument(err, skip(state), fields(otel.kind = "internal"))]
#[get("/api/v2/health")]
pub async fn get_health_v2(state: web::Data<GlobalState>) -> Result<HealthV2, APIError> {
    state
        .store
        .send(GetHealth {}.trace())
        .await?
        .map(|health| health.into())
}

#[cfg(test)]
mod tests {
    use crate::api::test::*;
    use crate::models::*;

    #[actix_rt::test]
    async fn health_v1() {
        test_log_init();

        let content: HealthV1 = test_request!(GET "/api/v1/health" => OK with content);
        assert!(content.ok);
    }

    #[actix_rt::test]
    async fn health_v2() {
        test_log_init();

        test_state!(state = []);

        let content: HealthV2 =
            test_request!(GET "/api/v2/health" => OK with content | state = state);
        assert!(content.ok);
        assert_eq!(
            content.started_at,
            state
                .store
                .send(GetHealth {})
                .await
                .expect("the actor should respond")
                .expect("we should get the health")
                .started_at
        );
    }
}
