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
            type Error = actix_web::Error;
            type Future = futures::future::Ready<Result<actix_web::HttpResponse, actix_web::Error>>;

            fn respond_to(self, _req: &actix_web::HttpRequest) -> Self::Future {
                futures::future::ready(Ok(
                    actix_web::HttpResponse::Ok()
                    .content_type("application/json")
                    .json(&self)))
            }
        }
    };

    ($t:ty => ($req:ident, $model:ident) -> $location:expr) => {
        impl actix_web::Responder for $t {
            type Error = actix_web::Error;
            type Future = futures::future::Ready<Result<actix_web::HttpResponse, actix_web::Error>>;

            fn respond_to(self, $req: &actix_web::HttpRequest) -> Self::Future {
                if $req.method() == http::Method::POST {
                    let $model = &self;
                    futures::future::ready(Ok(actix_web::HttpResponse::Created()
                        .content_type("application/json")
                        .header("Location", $location.expect("a location url").into_string())
                        .json(&self)))
                } else {                
                    futures::future::ready(Ok(actix_web::HttpResponse::Ok()
                        .content_type("application/json")
                        .json(&self)))
                }
            }
        }
    };
}