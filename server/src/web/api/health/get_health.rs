use actix_web::web;
use rex_api::{ApiError, HealthV1, HealthV2};
use tracing::instrument;

use crate::{db::Store, services::Services, web::ApiResponse};

#[instrument(err, skip(services), fields(otel.kind = "internal"))]
pub async fn get_health_v1<S: Services>(
    services: web::Data<S>,
) -> Result<ApiResponse<HealthV1>, ApiError> {
    services
        .store()
        .health()
        .await
        .map(|health| ApiResponse(health.into()))
}

#[instrument(err, skip(services), fields(otel.kind = "internal"))]
pub async fn get_health_v2<S: Services>(
    services: web::Data<S>,
) -> Result<ApiResponse<HealthV2>, ApiError> {
    services
        .store()
        .health()
        .await
        .map(|health| ApiResponse(health.into()))
}

#[cfg(test)]
mod tests {
    use rex_api::{HealthV1, HealthV2};

    use crate::{db::Store, test_request, test_state, testing::*};

    #[actix_rt::test]
    async fn health_v1() {
        test_log_init();

        let content: HealthV1 = test_request!(GET "/api/v1/health" => OK with content);
        assert!(content.ok);
    }

    #[actix_rt::test]
    async fn healthz_is_an_alias() {
        test_log_init();

        let content: HealthV1 = test_request!(GET "/healthz" => OK with content);
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
                .store()
                .health()
                .await
                .expect("we should get the health")
                .started_at
        );
    }
}
