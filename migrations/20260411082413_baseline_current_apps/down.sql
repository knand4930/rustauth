ALTER TABLE user.users DROP CONSTRAINT IF EXISTS uq_users_email;

DROP INDEX IF EXISTS user.idx_users_is_active;

DROP TABLE IF EXISTS user.users CASCADE;

ALTER TABLE user.refresh_tokens DROP CONSTRAINT IF EXISTS uq_refresh_tokens_refresh_token;

DROP INDEX IF EXISTS user.idx_refresh_tokens_user_id_is_active;

DROP TABLE IF EXISTS user.refresh_tokens CASCADE;

ALTER TABLE user.access_tokens DROP CONSTRAINT IF EXISTS uq_access_tokens_access_token;

DROP INDEX IF EXISTS user.idx_access_tokens_user_id_is_active;

DROP TABLE IF EXISTS user.access_tokens CASCADE;

ALTER TABLE user.token_blacklists DROP CONSTRAINT IF EXISTS uq_token_blacklists_token_jti;

DROP TABLE IF EXISTS user.token_blacklists CASCADE;

ALTER TABLE user.password_reset_tokens DROP CONSTRAINT IF EXISTS uq_password_reset_tokens_token_hash;

DROP TABLE IF EXISTS user.password_reset_tokens CASCADE;

ALTER TABLE user.user_sessions DROP CONSTRAINT IF EXISTS uq_user_sessions_session_token;

DROP INDEX IF EXISTS user.idx_user_sessions_user_id_is_active;

DROP TABLE IF EXISTS user.user_sessions CASCADE;

ALTER TABLE user.permissions DROP CONSTRAINT IF EXISTS uq_permissions_name;

DROP TABLE IF EXISTS user.permissions CASCADE;

ALTER TABLE user.roles DROP CONSTRAINT IF EXISTS uq_roles_name;

DROP TABLE IF EXISTS user.roles CASCADE;

ALTER TABLE user.role_permissions DROP CONSTRAINT IF EXISTS uq_role_permissions_role_id_permission_id;

DROP TABLE IF EXISTS user.role_permissions CASCADE;

ALTER TABLE blogs.blog_posts DROP CONSTRAINT IF EXISTS uq_blog_posts_slug;

DROP INDEX IF EXISTS blogs.idx_blog_posts_author_id_is_published;

DROP TABLE IF EXISTS blogs.blog_posts CASCADE;

DROP INDEX IF EXISTS blogs.idx_comments_blog_post_id_is_approved;

DROP TABLE IF EXISTS blogs.comments CASCADE;

ALTER TABLE user.user_roles DROP CONSTRAINT IF EXISTS uq_user_roles_user_id_role_id;

DROP INDEX IF EXISTS user.idx_user_roles_role_id_is_active;

DROP TABLE IF EXISTS user.user_roles CASCADE;