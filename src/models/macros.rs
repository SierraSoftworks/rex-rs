#[macro_export]
macro_rules! actor_message {
    ($t:ident ( $($pn:ident: $pt:ty),* ) -> $rt:ty) => {
        #[derive(Debug, Default)]
        pub struct $t {
            $(pub $pn: $pt),*
        }

        impl Message for $t {
            type Result = Result<$rt, APIError>;
        }
    };
}

#[macro_export]
macro_rules! json_responder {
    ($t:ty) => {
        impl actix_web::Responder for $t {
            type Body = actix_web::body::BoxBody;

            #[tracing::instrument(target="response.render", fields(http.content_type = "application/json"), skip(self, _req))]
            fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
                    actix_web::HttpResponse::Ok()
                    .content_type("application/json")
                    .json(&self)
            }
        }
    };

    ($t:ty => ($req:ident, $model:ident) -> $location:expr) => {
        impl actix_web::Responder for $t {
            type Body = actix_web::body::BoxBody;

            #[tracing::instrument(target="response.render", fields(http.content_type = "application/json"), skip(self, $req))]
            fn respond_to(self, $req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
                if $req.method() == http::Method::POST {
                    let $model = &self;
                    actix_web::HttpResponse::Created()
                        .content_type("application/json")
                        .insert_header(("Location", String::from($location.expect("a location url"))))
                        .json(&self)
                } else {
                    actix_web::HttpResponse::Ok()
                        .content_type("application/json")
                        .json(&self)
                }
            }
        }
    };
}
