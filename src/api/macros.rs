#[macro_export]
macro_rules! parse_uuid {
    ($from:expr, $desc:expr) => {
        u128::from_str_radix($from.replace("-", "").as_str(), 16)
            .or(Err(APIError::new(400, "Bad Request", &format!("The {} you provided could not be parsed. Please check it and try again.", $desc))))?
    };
}

#[macro_export]
macro_rules! require_role {
    (_cond: $r:ident ->) => (());
    (_cond: $r:ident -> $role:expr) => {
        $r == $role
    };

    (_cond: $r:ident -> $role:expr, $($rest:expr),+) => {
        $r == $role || require_role!(_cond: $r -> $($rest),+)
    };

    ($token:expr, $($role:expr),+) => {
        if !$token.roles().iter().any(|r| require_role!(_cond: r -> $($role),*)) {
            return Err(APIError::new(403, "Forbidden", "You are not authorized to perform this action. Please contact your administrator for permission before trying again."));
        }
    };
}

#[macro_export]
macro_rules! require_scope {
    ($token:expr, $scope:expr) => {
        if !$token.scopes().iter().any(|&s| s == $scope) {
            return Err(APIError::new(403, "Forbidden", "Your client has not been granted permission to access this resource. Please request the necessary access scopes and try again."));
        }
    };
}

#[macro_export]
macro_rules! test_state {
    (:: $state:ident = []) => (());
    (:: $state:ident = [ $item:expr ]) => {
        $state.store.send($item).await.expect("the actor should be run").expect("the operation should succeed");
    };

    (:: $state:ident = [ $item:expr, $($rest:expr),* ]) => {
        test_state!(:: $state = [ $item ]);
        test_state!(:: $state = [ $($rest),* ]);
    };

    ($state:ident = [ $($init:expr),* ]) => {
        let $state = crate::models::GlobalState::new();

        test_state!(:: $state = [ $($init),* ]);
    }
}

#[macro_export]
macro_rules! test_request {

    ($method:ident $path:expr => $status:ident | state = $state:ident) => {
        {
            let mut app = crate::api::test::get_test_app($state.clone()).await;
            let req = actix_web::test::TestRequest::with_uri($path)
                .method(http::Method::$method)
                .insert_header(("Authorization", crate::api::test::auth_token()))
                .to_request();

            let mut response = actix_web::test::call_service(&mut app, req).await;
            crate::api::test::assert_status(&mut response, http::StatusCode::$status).await;

            response
        }
    };

    ($method:ident $path:expr, $body:expr => $status:ident | state = $state:ident) => {
        {
            let mut app = crate::api::test::get_test_app($state.clone()).await;
            let req = actix_web::test::TestRequest::with_uri($path)
                .method(http::Method::$method)
                .set_json(&$body)
                .insert_header(("Authorization", crate::api::test::auth_token()))
                .to_request();

            let mut response = actix_web::test::call_service(&mut app, req).await;
            crate::api::test::assert_status(&mut response, http::StatusCode::$status).await;

            response
        }
    };

    ($method:ident $path:expr => $status:ident with content | state = $state:ident) => {
        {
            let mut response = test_request!($method $path => $status | state = $state);
            crate::api::test::get_content(&mut response).await
        }
    };

    ($method:ident $path:expr, $body:expr => $status:ident with content | state = $state:ident) => {
        {
            let mut response = test_request!($method $path, $body => $status | state = $state);
            crate::api::test::get_content(&mut response).await
        }
    };

    ($method:ident $path:expr => $status:ident with location =~ $location:expr, content | state = $state:ident) => {
        {
            let mut response = test_request!($method $path => $status | state = $state);
            crate::api::test::assert_location_header(response.headers(), $location);
            crate::api::test::get_content(&mut response).await
        }
    };

    ($method:ident $path:expr, $body:expr => $status:ident with location =~ $location:expr, content | state = $state:ident) => {
        {
            let mut response = test_request!($method $path, $body => $status | state = $state);
            crate::api::test::assert_location_header(response.headers(), $location);
            crate::api::test::get_content(&mut response).await
        }
    };

    /* --------------- NO GLOBAL STATE ------------------ */
    
    ($method:ident $path:expr => $status:ident) => {
        {
            let state = crate::models::GlobalState::new();
            
            test_request!($method $path => $status | state = state)
        }
    };

    ($method:ident $path:expr, $body:expr => $status:ident) => {
        {
            let state = crate::models::GlobalState::new();
            
            test_request!($method $path, $body => $status | state = state)
        }
    };

    ($method:ident $path:expr => $status:ident with content) => {
        {
            let mut response = test_request!($method $path => $status);
            crate::api::test::get_content(&mut response).await
        }
    };

    ($method:ident $path:expr, $body:expr => $status:ident with content) => {
        {
            let mut response = test_request!($method $path, $body => $status);
            crate::api::test::get_content(&mut response).await
        }
    };

    ($method:ident $path:expr => $status:ident with location =~ $location:expr, content) => {
        {
            let mut response = test_request!($method $path => $status);
            assert_location_header(response.headers(), $location);
            crate::api::test::get_content(&mut response).await
        }
    };

    ($method:ident $path:expr, $body:expr => $status:ident with location =~ $location:expr, content) => {
        {
            let mut response = test_request!($method $path, $body => $status);
            assert_location_header(response.headers(), $location);
            crate::api::test::get_content(&mut response).await
        }
    };
}