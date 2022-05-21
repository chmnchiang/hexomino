CREATE TABLE Users (
  id uuid NOT NULL PRIMARY KEY default gen_random_uuid(),
  username TEXT NOT NULL,
  name TEXT,
  password TEXT NOT NULL
);

CREATE UNIQUE INDEX users_name_index
ON Users(username);
