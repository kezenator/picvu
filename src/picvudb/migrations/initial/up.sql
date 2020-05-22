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
  obj_type TEXT NOT NULL,
  title TEXT
);

CREATE INDEX objects_by_changed_timestamp
  ON objects(changed_timestamp, id);

CREATE TABLE attachments_metadata (
  obj_id TEXT NOT NULL PRIMARY KEY,
  filename TEXT NOT NULL,
  created INTEGER NOT NULL,
  modified INTEGER NOT NULL,
  mime TEXT NOT NULL,
  size INTEGER NOT NULL,
  hash TEXT NOT NULL
);

CREATE TABLE attachments_data (
  obj_id TEXT NOT NULL,
  offset BIGINT NOT NULL,
  bytes BLOB NOT NULL,
  UNIQUE(obj_id, offset)
);

CREATE INDEX attachments_metadata_by_filename
  ON attachments_metadata(filename, obj_id);

CREATE INDEX attachments_metadata_by_size
  ON attachments_metadata(size, obj_id);

CREATE INDEX attachments_metadata_by_hash
  ON attachments_metadata(hash, obj_id);

create INDEX attachments_data_by_obj_id_offset
  ON attachments_data(obj_id, offset);
