// startapp:modules:start
pub mod blogs;
pub mod user;
// startapp:modules:end

use axum::Router;
use utoipa::OpenApi;

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "RustAuth API",
        version = "1.0.0",
        description = "Authentication, Blog, and AdminX API built with Axum + SQLx",
    ),
    paths(
        crate::health_check,
        crate::admin::resource::dashboard::dashboard,
        crate::admin::resource::registry::list_resources,
        crate::admin::resource::registry::get_app_resources,
        crate::admin::resource::users::list_users,
        crate::admin::resource::users::get_user,
        crate::admin::resource::users::update_user,
        user::handlers::register,
        user::handlers::login,
        user::handlers::list_users,
        user::handlers::get_user,
        user::handlers::update_user,
        user::handlers::delete_user,
        blogs::handlers::create_blog_post,
        blogs::handlers::list_blog_posts,
        blogs::handlers::get_blog_post,
        blogs::handlers::update_blog_post,
        blogs::handlers::delete_blog_post,
        blogs::handlers::create_comment,
        blogs::handlers::list_comments,
    ),
    components(schemas(
        crate::admin::AdminEndpointConfig,
        crate::admin::AdminCrudConfig,
        crate::admin::AdminResourceConfig,
        crate::admin::AdminAppConfig,
        crate::admin::AdminExtension,
        crate::admin::AdminPanel,
        crate::admin::resource::dashboard::AdminDashboardResponse,
        crate::admin::resource::users::AdminUserResponse,
        crate::admin::resource::users::UpdateAdminUserRequest,
        user::User,
        user::UserResponse,
        user::RegisterRequest,
        user::LoginRequest,
        user::UpdateUserRequest,
        user::AuthTokenResponse,
        blogs::BlogPost,
        blogs::Comment,
        blogs::CreateBlogPostRequest,
        blogs::UpdateBlogPostRequest,
        blogs::CreateCommentRequest,
    )),
    tags(
        (name = "System", description = "Health and system endpoints"),
        (name = "AdminX", description = "Admin panel registry, dashboard, and custom resources"),
        (name = "Authentication", description = "Register & Login endpoints"),
        (name = "Users", description = "User CRUD operations"),
        (name = "Blog Posts", description = "Blog post management"),
        (name = "Comments", description = "Blog post comments"),
    )
)]
pub struct ApiDoc;

pub fn routes() -> Router<AppState> {
    // startapp:routes:start
    let router = Router::new().merge(blogs::routes()).merge(user::routes());
    // startapp:routes:end

    router
}
