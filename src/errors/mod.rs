use rocket_contrib::json::Json;
use sentry::{capture_message, with_scope, Level, Scope};
use serde_json::json;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Error {
    pub code: u16,
    pub error: String,
    pub description: String,
}

#[catch(404)]
pub fn error_404(req: &rocket::Request) -> Json<Error> {
    with_scope(
        |scope: &mut Scope| {
            scope.set_extra("request", sentry_request(req));
            scope.set_extra(
                "route",
                req.route()
                    .map(|route| json!(format!("{} {}", route.method, route.uri)))
                    .unwrap_or(serde_json::Value::Null),
            );
        },
        || {
            capture_message(
                &req.route()
                    .map(|route| format!("404 Not Found - {} {}", route.method, route.uri))
                    .unwrap_or("404 Not Found".into()),
                Level::Warning,
            )
        },
    );

    Json(Error{
        code: 404,
        error: "Not Found".into(),
        description: "The resource you requested could not be found, please check your request and try again.".into(),
    })
}

#[catch(422)]
pub fn error_422(req: &rocket::Request) -> Json<Error> {
    with_scope(
        |scope: &mut Scope| {
            scope.set_extra("request", sentry_request(req));
            scope.set_extra(
                "route",
                req.route()
                    .map(|route| json!(format!("{} {}", route.method, route.uri)))
                    .unwrap_or(serde_json::Value::Null),
            );
        },
        || {
            capture_message(
                &req.route()
                    .map(|route| {
                        format!("422 Unprocessable Entity - {} {}", route.method, route.uri)
                    })
                    .unwrap_or("422 Unprocessable Entity".into()),
                Level::Warning,
            )
        },
    );

    Json(Error{
        code: 422,
        error: "Unprocessable Entity".into(),
        description: "The request you submitted could not be processed according to the required schema. Please check your request and try again.".into(),
    })
}

#[catch(500)]
pub fn error_500(req: &rocket::Request) -> Json<Error> {
    with_scope(
        |scope: &mut Scope| {
            scope.set_extra("request", sentry_request(req));
            scope.set_extra(
                "route",
                req.route()
                    .map(|route| json!(format!("{} {}", route.method, route.uri)))
                    .unwrap_or(serde_json::Value::Null),
            );
        },
        || {
            capture_message(
                &req.route()
                    .map(|route| {
                        format!("500 Internal Server Error - {} {}", route.method, route.uri)
                    })
                    .unwrap_or("500 Internal Server Error".into()),
                Level::Warning,
            )
        },
    );

    Json(Error {
        code: 500,
        error: "Internal Server Error".into(),
        description:
            "We encountered an error while processing your request, please try again later.".into(),
    })
}

fn sentry_request(req: &rocket::Request) -> serde_json::Value {
    json!({
        "url": format!("{}", req.uri()),
        "method": format!("{}", req.method()),
        "cookies": req.cookies().iter().fold(String::new(), |mut s, cookie| {
            s.push_str(&format!("{}", cookie));
            s
        }),
        "headers": req.headers().iter().fold(HashMap::new(), |mut map, header| {
            map.insert(header.name().to_string(), header.value.to_string());
            map
        }),
    })
}
