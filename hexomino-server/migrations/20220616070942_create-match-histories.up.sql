CREATE TABLE MatchHistories(
  id uuid NOT NULL PRIMARY KEY,
  users uuid ARRAY[2] NOT NULL,
  scores integer ARRAY[2] NOT NULL,
  end_time TIMESTAMP WITH TIME ZONE NOT NULL,
  config TEXT,
  match_token TEXT,
  game_histories uuid ARRAY NOT NULL
);

CREATE INDEX match_history_time_index
ON MatchHistories(end_time);

CREATE TABLE GameHistories(
  id uuid NOT NULL PRIMARY KEY default gen_random_uuid(),
  match_id uuid NOT NULL,
  user_player_is_swapped boolean NOT NULL,
  winner_is_first_player boolean NOT NULL,
  actions_json TEXT NOT NULL
);

CREATE INDEX game_histories_match_index
ON GameHistories(match_id);

CREATE TABLE UserHistories(
  id serial NOT NULL PRIMARY KEY,
  user_id uuid NOT NULL,
  match_id uuid NOT NULL
);
