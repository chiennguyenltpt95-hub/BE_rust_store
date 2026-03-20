-- Migration: create users table

CREATE TYPE user_role AS ENUM ('admin', 'customer', 'seller');
CREATE TYPE user_status AS ENUM ('active', 'inactive', 'banned');

CREATE TABLE IF NOT EXISTS users (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email        VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    full_name    VARCHAR(100) NOT NULL,
    role         user_role NOT NULL DEFAULT 'customer',
    status       user_status NOT NULL DEFAULT 'active',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_status ON users(status);


Up
