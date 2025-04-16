CREATE DATABASE tasks_db;

CREATE TYPE task_status AS ENUM
('not_started', 'in_progress', 'complete', 'cancelled', 'blocked');

CREATE TABLE tasks (
    id uuid PRIMARY KEY,
    title varchar(64) NOT NULL,
    -- note that description *is* nullable since it's optional
    description text,
    status task_status NOT NULL,
    due timestamp NOT NULL
);
