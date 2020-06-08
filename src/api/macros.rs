#[macro_export]
macro_rules! parse_uuid {
    ($from:expr, $($desc:tt)+) => {
        u128::from_str_radix($from.replace("-", "").as_str(), 16)
            .or(Err(APIError::new(400, "Bad Request", "The $($desc)+ you provided could not be parsed. Please check it and try again.")))?;
    };
}

#[macro_export]
macro_rules! require_role{
    (_cond: $r:ident ->) => (());
    (_cond: $r:ident -> $role:expr) => {
        $r == $role
    };

    (_cond: $r:ident -> $role:expr, $($rest:expr),+) => {
        $r == $role || require_role!(_cond: $r -> $($rest),+)
    };

    ($token:expr, $($role:expr),+) => {
        if !$token.roles.iter().any(|r| require_role!(_cond: r -> $($role),*)) {
            return Err(APIError::new(403, "Forbidden", "You are not authorized to perform this action. Please contact your administrator for permission before trying again."));
        }
    };
}

#[macro_export]
macro_rules! require_scope{
    ($token:expr, $scope:expr) => {
        if !$token.scp.split(" ").any(|s| s == $scope) {
            return Err(APIError::new(403, "Forbidden", "Your client has not been granted permission to access this resource. Please request the necessary access scopes and try again."));
        }
    };
}