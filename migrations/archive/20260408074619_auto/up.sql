CREATE SCHEMA IF NOT EXISTS activitylog;

CREATE SCHEMA IF NOT EXISTS blogs;

CREATE SCHEMA IF NOT EXISTS products;

CREATE SCHEMA IF NOT EXISTS user;

CREATE TABLE blogs.blog_posts (
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

CREATE TABLE user.users (
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

CREATE TABLE user.refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    refresh_token VARCHAR NOT NULL,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL,
    user_agent VARCHAR,
    ip_address VARCHAR,
    device_fingerprint VARCHAR,
    last_used_at TIMESTAMPTZ,
    rotated_from_id UUID,
    user_id UUID NOT NULL REFERENCES user.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user.access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user.users(id),
    refresh_token_id UUID NOT NULL REFERENCES user.refresh_tokens(id),
    access_token VARCHAR NOT NULL,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL,
    last_used TIMESTAMPTZ,
    is_single_use BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user.token_blacklists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_jti VARCHAR,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user.password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES user.users(id),
    token_hash VARCHAR,
    expires_at TIMESTAMPTZ,
    is_used BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user.user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES user.users(id),
    session_token VARCHAR,
    user_agent VARCHAR,
    ip_address VARCHAR,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user.permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR,
    is_active BOOLEAN,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user.roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description VARCHAR,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user.role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES user.roles(id),
    permission_id UUID NOT NULL REFERENCES user.permissions(id),
    can_read BOOLEAN NOT NULL,
    can_write BOOLEAN NOT NULL,
    can_delete BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE products.products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE activitylog.activity_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES user.users(id),
    action VARCHAR NOT NULL,
    entity VARCHAR,
    entity_id UUID,
    status VARCHAR NOT NULL,
    message VARCHAR,
    ip_address VARCHAR,
    user_agent VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE blogs.comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES user.users(id),
    guest_name VARCHAR,
    blog_post_id UUID NOT NULL REFERENCES blogs.blog_posts(id),
    parent_id UUID,
    content VARCHAR NOT NULL,
    is_approved BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user.user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user.users(id),
    role_id UUID NOT NULL REFERENCES user.roles(id),
    assigned_by_id UUID,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);