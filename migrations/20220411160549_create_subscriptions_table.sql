-- Add migration script here
create table subscriptions(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    name TEXT NOT NULL,
    email TEXT NOT NULL unique,
    subscribed_at timestamptz NOT NULL
);