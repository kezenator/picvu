CREATE TABLE db_properties (
  name TEXT NOT NULL PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE objects (
  id TEXT NOT NULL PRIMARY KEY,
  created_timestamp INTEGER NOT NULL,
  created_offset INTEGER NOT NULL,
  modified_timestamp INTEGER NOT NULL,
  modified_offset INTEGER NOT NULL,
  activity_timestamp INTEGER NOT NULL,
  activity_offset INTEGER NOT NULL,
  obj_type TEXT NOT NULL,
  title TEXT,
  notes TEXT,
  rating INTEGER,
  censor INTEGER NOT NULL,
  latitude REAL,
  longitude REAL
);

CREATE INDEX objects_by_modified_timestamp
  ON objects(modified_timestamp, id);

CREATE INDEX objects_by_activity_timestamp
  ON objects(activity_timestamp, id);

CREATE TABLE attachments_metadata (
  obj_id TEXT NOT NULL PRIMARY KEY,
  filename TEXT NOT NULL,
  created_timestamp INTEGER NOT NULL,
  created_offset INTEGER NOT NULL,
  modified_timestamp INTEGER NOT NULL,
  modified_offset INTEGER NOT NULL,
  mime TEXT NOT NULL,
  size INTEGER NOT NULL,
  orientation INTEGER,
  width INTEGER,
  height INTEGER,
  duration INTEGER,
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
