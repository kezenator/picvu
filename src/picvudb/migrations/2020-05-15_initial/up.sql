CREATE TABLE db_properties (
  name TEXT NOT NULL PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE objects (
  id TEXT NOT NULL PRIMARY KEY,
  added_timestamp INTEGER NOT NULL,
  added_timestring TEXT NOT NULL,
  changed_timestamp INTEGER NOT NULL,
  changed_timestring TEXT NOT NULL,
  label TEXT NOT NULL
);

CREATE INDEX objects_by_changed_timestamp
  ON objects(changed_timestamp, id);
