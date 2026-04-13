CREATE SCHEMA IF NOT EXISTS activity;

CREATE SCHEMA IF NOT EXISTS auth;

CREATE SCHEMA IF NOT EXISTS blog;

CREATE TABLE auth.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR,
    password VARCHAR,
    store_password VARCHAR,
    full_name VARCHAR,
    company VARCHAR,
    avatar_url VARCHAR,
    phone_number VARCHAR,
    timezone VARCHAR NOT NULL,
    language VARCHAR NOT NULL,
    salt VARCHAR,
    location VARCHAR,
    ipaddress VARCHAR,
    is_active BOOLEAN NOT NULL,
    is_superuser BOOLEAN NOT NULL,
    is_staffuser BOOLEAN NOT NULL,
    is_guest BOOLEAN,
    email_verified BOOLEAN NOT NULL,
    phone_verified BOOLEAN NOT NULL,
    mfa_enabled BOOLEAN NOT NULL,
    mfa_secret VARCHAR,
    backup_codes TEXT[],
    preferences JSONB,
    last_login_at TIMESTAMPTZ,
    last_login_ip VARCHAR,
    login_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE auth.refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    refresh_token VARCHAR NOT NULL,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL,
    user_agent VARCHAR,
    ip_address VARCHAR,
    device_fingerprint VARCHAR,
    last_used_at TIMESTAMPTZ,
    rotated_from_id UUID,
    user_id UUID NOT NULL REFERENCES auth.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE auth.access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id),
    refresh_token_id UUID NOT NULL REFERENCES auth.refresh_tokens(id),
    access_token VARCHAR NOT NULL,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL,
    last_used TIMESTAMPTZ,
    is_single_use BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE auth.token_blacklists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_jti VARCHAR,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE auth.password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES auth.users(id),
    token_hash VARCHAR,
    expires_at TIMESTAMPTZ,
    is_used BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE auth.user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES auth.users(id),
    session_token VARCHAR,
    user_agent VARCHAR,
    ip_address VARCHAR,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE auth.permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR,
    is_active BOOLEAN,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE auth.roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description VARCHAR,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE auth.role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES auth.roles(id),
    permission_id UUID NOT NULL REFERENCES auth.permissions(id),
    can_read BOOLEAN NOT NULL,
    can_write BOOLEAN NOT NULL,
    can_delete BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE blog.blog_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR NOT NULL,
    slug VARCHAR NOT NULL,
    author_id UUID NOT NULL,
    content VARCHAR NOT NULL,
    short_description VARCHAR NOT NULL,
    is_published BOOLEAN NOT NULL,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE blog.comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES auth.users(id),
    guest_name VARCHAR,
    blog_post_id UUID NOT NULL REFERENCES blog.blog_posts(id),
    parent_id UUID,
    content VARCHAR NOT NULL,
    is_approved BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE activity.activity_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES auth.users(id),
    action VARCHAR NOT NULL,
    entity VARCHAR,
    entity_id UUID,
    status VARCHAR NOT NULL,
    message VARCHAR,
    ip_address VARCHAR,
    user_agent VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE auth.user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id),
    role_id UUID NOT NULL REFERENCES auth.roles(id),
    assigned_by_id UUID,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);