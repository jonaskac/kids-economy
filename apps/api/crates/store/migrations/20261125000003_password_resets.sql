-- Password reset links (ADR-0012). An Owner makes one for a Member; only the SHA-256 of
-- the link secret is stored. Used once, within 24 hours.
create table password_resets (
    token_hash  text        primary key,
    user_id     uuid        not null references documents (id),
    created_at  timestamptz not null default now(),
    expires_at  timestamptz not null,
    used_at     timestamptz
);

create index password_resets_by_user on password_resets (user_id);
