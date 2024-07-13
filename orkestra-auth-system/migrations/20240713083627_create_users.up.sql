-- Add up migration script here
create table if not exists "users"
(
    id uuid primary key default gen_random_uuid(),
    username text unique not null,
    password varchar not null
);