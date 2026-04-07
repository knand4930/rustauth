DROP TABLE IF EXISTS auth.users CASCADE;

DROP TABLE IF EXISTS auth.refresh_tokens CASCADE;

DROP TABLE IF EXISTS auth.access_tokens CASCADE;

DROP TABLE IF EXISTS auth.token_blacklists CASCADE;

DROP TABLE IF EXISTS auth.password_reset_tokens CASCADE;

DROP TABLE IF EXISTS auth.user_sessions CASCADE;

DROP TABLE IF EXISTS auth.permissions CASCADE;

DROP TABLE IF EXISTS auth.roles CASCADE;

DROP TABLE IF EXISTS auth.role_permissions CASCADE;

DROP TABLE IF EXISTS blog.blog_posts CASCADE;

DROP TABLE IF EXISTS blog.comments CASCADE;

DROP TABLE IF EXISTS activity.activity_logs CASCADE;

DROP TABLE IF EXISTS auth.user_roles CASCADE;