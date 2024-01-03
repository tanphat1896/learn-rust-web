-- Add up migration script here
create table if not exists account (
  id serial primary key,
  email varchar(255) unique not null,
  password varchar(255) not null,
  created_at timestamp not null default now()
);
