-- Add up migration script here


create table if not exists answers(
  id serial primary key,
  content text not null,
  corresponding_question integer references questions,
  created_at timestamp not null default now()
);