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
  title TEXT
);

CREATE INDEX objects_by_changed_timestamp
  ON objects(changed_timestamp, id);

CREATE TABLE attachments_metadata (
  id TEXT NOT NULL PRIMARY KEY,
  filename TEXT NOT NULL,
  created INTEGER NOT NULL,
  modified INTEGER NOT NULL,
  mime TEXT NOT NULL,
  size INTEGER NOT NULL,
  hash TEXT NOT NULL
);

CREATE TABLE attachments_data (
  id TEXT NOT NULL PRIMARY KEY,
  bytes BLOB NOT NULL
);

CREATE INDEX attachments_metadata_by_filename
  ON attachments_metadata(filename, id);

CREATE INDEX attachments_metadata_by_size
  ON attachments_metadata(size, id);

CREATE INDEX attachments_metadata_by_hash
  ON attachments_metadata(hash, id);
