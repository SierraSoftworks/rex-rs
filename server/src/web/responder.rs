use actix_web::{HttpRequest, HttpResponse, Responder, body::BoxBody, http::Method};
use rex_api::{CollectionV3, IdeaV1, IdeaV2, IdeaV3, RoleAssignmentV3, UserV3};
use serde::Serialize;
use tracing::warn;

/// Wraps a wire type on its way out of a handler.
///
/// The wire types live in `rex-api`, which knows nothing about actix, so the
/// `Responder` implementation lives here instead — on a local wrapper, with the
/// per-type "where does this resource live" knowledge expressed through the
/// local [`ApiLocation`] trait.
pub struct ApiResponse<T>(pub T);

/// How to address a resource, so that a `POST` which creates one can say where
/// it went.
pub trait ApiLocation {
    fn location(&self, req: &HttpRequest) -> Option<String>;
}

impl<T: Serialize + ApiLocation> Responder for ApiResponse<T> {
    type Body = BoxBody;

    #[tracing::instrument(target = "response.render", fields(http.content_type = "application/json"), skip(self, req))]
    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        if req.method() != Method::POST {
            return HttpResponse::Ok()
                .content_type("application/json")
                .json(&self.0);
        }

        match self.0.location(req) {
            Some(location) => HttpResponse::Created()
                .content_type("application/json")
                .insert_header(("Location", location))
                .json(&self.0),
            None => {
                // Worth complaining about: a 201 without a Location is a
                // contract violation, but it is not worth failing the write the
                // caller already made.
                warn!("We created a resource but could not work out its location.");
                HttpResponse::Created()
                    .content_type("application/json")
                    .json(&self.0)
            }
        }
    }
}

fn url_for(req: &HttpRequest, name: &str, parts: Vec<String>) -> Option<String> {
    req.url_for(name, parts)
        .map(|url| url.to_string())
        .map_err(|err| {
            warn!("We could not build a URL for `{}`: {}", name, err);
            err
        })
        .ok()
}

impl ApiLocation for IdeaV1 {
    fn location(&self, req: &HttpRequest) -> Option<String> {
        url_for(req, "get_idea_v1", vec![self.id.clone()?])
    }
}

impl ApiLocation for IdeaV2 {
    fn location(&self, req: &HttpRequest) -> Option<String> {
        url_for(req, "get_idea_v2", vec![self.id.clone()?])
    }
}

impl ApiLocation for IdeaV3 {
    fn location(&self, req: &HttpRequest) -> Option<String> {
        // The same idea is addressable both inside and outside a collection;
        // point at whichever form the caller used.
        if req.uri().path().contains("/collection/") {
            url_for(
                req,
                "get_collection_idea_v3",
                vec![self.collection.clone()?, self.id.clone()?],
            )
        } else {
            url_for(req, "get_idea_v3", vec![self.id.clone()?])
        }
    }
}

impl ApiLocation for CollectionV3 {
    fn location(&self, req: &HttpRequest) -> Option<String> {
        url_for(req, "get_collection_v3", vec![self.id.clone()?])
    }
}

impl ApiLocation for RoleAssignmentV3 {
    fn location(&self, req: &HttpRequest) -> Option<String> {
        url_for(
            req,
            "get_role_assignment_v3",
            vec![self.collection_id.clone()?, self.user_id.clone()?],
        )
    }
}

impl ApiLocation for UserV3 {
    fn location(&self, req: &HttpRequest) -> Option<String> {
        url_for(req, "get_user_v3", vec![self.email_hash.clone()])
    }
}

/// Health has no address of its own; it is only ever read.
macro_rules! no_location {
    ($($t:ty),+) => {
        $(impl ApiLocation for $t {
            fn location(&self, _req: &HttpRequest) -> Option<String> {
                None
            }
        })+
    };
}

no_location!(
    rex_api::HealthV1,
    rex_api::HealthV2,
    rex_api::AuthPrincipal,
    rex_api::AuthMetadata,
    rex_api::TokenResponse
);
