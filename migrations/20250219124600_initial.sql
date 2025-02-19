CREATE TABLE things
(
    id          UUID        NOT NULL PRIMARY KEY DEFAULT uuid_generate_v7(),
    name        TEXT        NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL             DEFAULT NOW()
)
