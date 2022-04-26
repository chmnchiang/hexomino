CREATE TABLE Users (
  id uuid NOT NULL PRIMARY KEY default gen_random_uuid(),
  name TEXT NOT NULL,
  password TEXT NOT NULL
);

CREATE UNIQUE INDEX users_name_index
ON Users(name);
