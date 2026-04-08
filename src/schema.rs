pub mod activity {
    include!("activitylog/schemas.rs");
}

pub mod auth {
    include!("user/schemas.rs");
}

pub mod blog {
    include!("blogs/schemas.rs");
}

pub mod products {
    include!("products/schemas.rs");
}

pub mod newsite {
    include!("newsite/schemas.rs");
}
