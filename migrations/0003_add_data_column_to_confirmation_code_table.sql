create table updates_data (
    confirmation_id uuid primary key references confirmation_codes(id),
    data json default null
);