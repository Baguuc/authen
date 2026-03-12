create table users (
    id uuid primary key,
    email text not null unique,
    password_hash text not null,
    created_at timestamp default current_timestamp,
    active boolean default false
);

create table registration_codes (
    id uuid primary key,
    code text not null,
    user_id uuid not null,
    foreign key(user_id) references users(id)
);