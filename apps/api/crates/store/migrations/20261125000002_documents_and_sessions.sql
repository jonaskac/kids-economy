-- Entities as current-state documents (ADR-0005, ADR-0010).
create table documents (
    id          uuid        primary key,
    kind        text        not null,          -- 'user' | 'family'
    family_id   uuid,                          -- tenant key; null for Users
    version     bigint      not null check (version > 0),
    body        jsonb       not null,          -- {"v": 1, …}
    created_at  timestamptz not null default now(),
    updated_at  timestamptz not null default now()
);

create index documents_by_family on documents (family_id) where family_id is not null;

-- One User per email address (stored trimmed and lower-cased by the domain).
create unique index documents_user_email on documents ((body ->> 'email')) where kind = 'user';

-- Finding Families by Member (user_id) or by Invitation link hash with @>.
create index documents_family_body on documents using gin (body jsonb_path_ops)
    where kind = 'family';

-- Signed-in devices (ADR-0011). Only the SHA-256 of the cookie secret is stored.
create table sessions (
    token_hash  text        primary key,
    user_id     uuid        not null references documents (id),
    created_at  timestamptz not null default now(),
    expires_at  timestamptz not null
);

create index sessions_by_user on sessions (user_id);
