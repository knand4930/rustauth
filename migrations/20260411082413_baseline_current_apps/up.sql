CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE SCHEMA IF NOT EXISTS blogs;

CREATE SCHEMA IF NOT EXISTS user;

CREATE TABLE IF NOT EXISTS user.users (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    email VARCHAR,
    password VARCHAR,
    store_password VARCHAR,
    full_name VARCHAR,
    company VARCHAR,
    avatar_url VARCHAR,
    phone_number VARCHAR,
    timezone VARCHAR NOT NULL DEFAULT 'UTC',
    language VARCHAR NOT NULL DEFAULT 'en',
    salt VARCHAR,
    location VARCHAR,
    ipaddress VARCHAR,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_superuser BOOLEAN NOT NULL DEFAULT false,
    is_staffuser BOOLEAN NOT NULL DEFAULT false,
    is_guest BOOLEAN DEFAULT false,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    phone_verified BOOLEAN NOT NULL DEFAULT false,
    mfa_enabled BOOLEAN NOT NULL DEFAULT false,
    mfa_secret VARCHAR,
    backup_codes TEXT[],
    preferences JSONB,
    last_login_at TIMESTAMPTZ,
    last_login_ip VARCHAR,
    login_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE user.users ADD CONSTRAINT uq_users_email UNIQUE (email); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE INDEX IF NOT EXISTS idx_users_is_active ON user.users (is_active);

CREATE TABLE IF NOT EXISTS user.refresh_tokens (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    refresh_token VARCHAR NOT NULL,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    user_agent VARCHAR,
    ip_address VARCHAR,
    device_fingerprint VARCHAR,
    last_used_at TIMESTAMPTZ,
    rotated_from_id UUID REFERENCES user.refresh_tokens(id),
    user_id UUID NOT NULL REFERENCES user.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE user.refresh_tokens ADD CONSTRAINT uq_refresh_tokens_refresh_token UNIQUE (refresh_token); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id_is_active ON user.refresh_tokens (user_id, is_active);

CREATE TABLE IF NOT EXISTS user.access_tokens (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user.users(id),
    refresh_token_id UUID NOT NULL REFERENCES user.refresh_tokens(id),
    access_token VARCHAR NOT NULL,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_used TIMESTAMPTZ,
    is_single_use BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE user.access_tokens ADD CONSTRAINT uq_access_tokens_access_token UNIQUE (access_token); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE INDEX IF NOT EXISTS idx_access_tokens_user_id_is_active ON user.access_tokens (user_id, is_active);

CREATE TABLE IF NOT EXISTS user.token_blacklists (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    token_jti VARCHAR,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE user.token_blacklists ADD CONSTRAINT uq_token_blacklists_token_jti UNIQUE (token_jti); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE TABLE IF NOT EXISTS user.password_reset_tokens (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES user.users(id),
    token_hash VARCHAR,
    expires_at TIMESTAMPTZ,
    is_used BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE user.password_reset_tokens ADD CONSTRAINT uq_password_reset_tokens_token_hash UNIQUE (token_hash); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE TABLE IF NOT EXISTS user.user_sessions (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES user.users(id),
    session_token VARCHAR,
    user_agent VARCHAR,
    ip_address VARCHAR,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE user.user_sessions ADD CONSTRAINT uq_user_sessions_session_token UNIQUE (session_token); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id_is_active ON user.user_sessions (user_id, is_active);

CREATE TABLE IF NOT EXISTS user.permissions (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    name VARCHAR,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE user.permissions ADD CONSTRAINT uq_permissions_name UNIQUE (name); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE TABLE IF NOT EXISTS user.roles (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description VARCHAR,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE user.roles ADD CONSTRAINT uq_roles_name UNIQUE (name); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE TABLE IF NOT EXISTS user.role_permissions (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES user.roles(id),
    permission_id UUID NOT NULL REFERENCES user.permissions(id),
    can_read BOOLEAN NOT NULL DEFAULT false,
    can_write BOOLEAN NOT NULL DEFAULT false,
    can_delete BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE user.role_permissions ADD CONSTRAINT uq_role_permissions_role_id_permission_id UNIQUE (role_id, permission_id); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE TABLE IF NOT EXISTS blogs.blog_posts (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    title VARCHAR NOT NULL,
    slug VARCHAR NOT NULL,
    author_id UUID NOT NULL REFERENCES user.users(id),
    content VARCHAR NOT NULL,
    short_description VARCHAR NOT NULL,
    is_published BOOLEAN NOT NULL DEFAULT false,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE blogs.blog_posts ADD CONSTRAINT uq_blog_posts_slug UNIQUE (slug); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE INDEX IF NOT EXISTS idx_blog_posts_author_id_is_published ON blogs.blog_posts (author_id, is_published);

CREATE TABLE IF NOT EXISTS blogs.comments (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES user.users(id),
    guest_name VARCHAR,
    blog_post_id UUID NOT NULL REFERENCES blogs.blog_posts(id),
    parent_id UUID REFERENCES blogs.comments(id),
    content VARCHAR NOT NULL,
    is_approved BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_comments_blog_post_id_is_approved ON blogs.comments (blog_post_id, is_approved);

CREATE TABLE IF NOT EXISTS user.user_roles (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user.users(id),
    role_id UUID NOT NULL REFERENCES user.roles(id),
    assigned_by_id UUID REFERENCES user.users(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$ BEGIN ALTER TABLE user.user_roles ADD CONSTRAINT uq_user_roles_user_id_role_id UNIQUE (user_id, role_id); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE INDEX IF NOT EXISTS idx_user_roles_role_id_is_active ON user.user_roles (role_id, is_active);