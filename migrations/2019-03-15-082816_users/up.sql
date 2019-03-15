create table users (
  id    serial  not null primary key,
  name  text    not null,
  admin boolean not null default false
);
