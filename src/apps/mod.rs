pub mod adminx;
pub mod blogs;
pub mod user;
// startapp:modules

use axum::Router;
use utoipa::OpenApi;

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "RustAuth API",
        version = "1.0.0",
        description = "Authentication & Blog API built with Axum + SQLx",
    ),
    paths(
        crate::health_check,
        adminx::handlers::dashboard,
        adminx::handlers::list_users,
        adminx::handlers::get_user,
        adminx::handlers::update_user,
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
        adminx::AdminDashboardResponse,
        adminx::AdminUserResponse,
        adminx::UpdateAdminUserRequest,
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
        (name = "AdminX", description = "Admin dashboard and user management"),
        (name = "Authentication", description = "Register & Login endpoints"),
        (name = "Users", description = "User CRUD operations"),
        (name = "Blog Posts", description = "Blog post management"),
        (name = "Comments", description = "Blog post comments"),
    )
)]
pub struct ApiDoc;

pub fn routes() -> Router<AppState> {
    let router = Router::new()
        .merge(adminx::routes())
        .merge(user::routes())
        .merge(blogs::routes());
    // startapp:routes

    router
}
