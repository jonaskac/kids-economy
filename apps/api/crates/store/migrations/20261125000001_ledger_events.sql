-- Wallet Histories: append-only, numbered per Wallet (ADR-0009).
create table ledger_events (
    family_id   uuid        not null,
    wallet_id   uuid        not null,
    sequence    bigint      not null check (sequence > 0),
    kind        text        not null,
    body        jsonb       not null,
    recorded_at timestamptz not null default now(),
    primary key (wallet_id, sequence)
);

-- Finding a Family's Wallets: one 'wallet_opened' row per Wallet.
create index ledger_events_family_wallets
    on ledger_events (family_id)
    where kind = 'wallet_opened';

-- Ledgers are append-only: mistakes get a Correction, never an edit.
create function ledger_events_append_only() returns trigger
language plpgsql as $$
begin
    raise exception 'ledger_events is append-only';
end;
$$;

create trigger ledger_events_append_only
    before update or delete on ledger_events
    for each row execute function ledger_events_append_only();

create trigger ledger_events_no_truncate
    before truncate on ledger_events
    for each statement execute function ledger_events_append_only();
