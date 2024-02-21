create type user_role_t as enum ('ADMIN', 'GUEST', 'USER');
create type rate_kind_t as enum ('LIKE', 'DISLIKE');

create table if not exists users
(
    id         serial primary key,
    role       user_role_t not null,
    first_name text        not null,
    last_name  text        not null
);

create table if not exists posts
(
    id                serial primary key,
    created_at        timestamp default now() not null,
    text              text                    not null,
    image_url         text                    not null,
    likes_quantity    integer   default 0     not null,
    dislikes_quantity integer   default 0     not null
);

create table if not exists user_credentials
(
    user_id       integer not null primary key references users,
    login         text    not null unique,
    password_hash bytea   not null
);

create table if not exists user_auth
(
    user_id integer not null primary key references users,
    token   text unique
);


create table if not exists post_rates
(
    post_id integer     not null references posts on delete cascade,
    user_id integer     not null references users on delete cascade,
    rate    rate_kind_t not null,
    primary key (post_id, user_id)
);


