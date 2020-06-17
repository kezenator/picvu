CREATE TABLE db_properties (
  name TEXT NOT NULL PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE objects (
  id INTEGER PRIMARY KEY,
  created_timestamp INTEGER NOT NULL,
  created_offset INTEGER,
  modified_timestamp INTEGER NOT NULL,
  modified_offset INTEGER,
  activity_timestamp INTEGER NOT NULL,
  activity_offset INTEGER,
  title TEXT,
  notes TEXT,
  rating INTEGER,
  censor INTEGER NOT NULL,
  latitude REAL,
  longitude REAL,
  tag_set TEXT
);

CREATE INDEX objects_by_modified_timestamp
  ON objects(modified_timestamp, id);

CREATE INDEX objects_by_activity_timestamp
  ON objects(activity_timestamp, id);

CREATE TABLE attachments_metadata (
  obj_id INTEGER PRIMARY KEY,
  filename TEXT NOT NULL,
  created_timestamp INTEGER NOT NULL,
  created_offset INTEGER,
  modified_timestamp INTEGER NOT NULL,
  modified_offset INTEGER,
  mime TEXT NOT NULL,
  size INTEGER NOT NULL,
  orientation INTEGER,
  width INTEGER,
  height INTEGER,
  duration INTEGER,
  hash TEXT NOT NULL
);

CREATE TABLE attachments_data (
  obj_id INTEGER NOT NULL,
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

CREATE VIRTUAL TABLE objects_fts USING fts5(
  id,
  title,
  notes,
  tokenize = 'porter unicode61',
  prefix = 3,
  content = objects,
  content_rowid = id);

CREATE VIRTUAL TABLE objects_location USING rtree(
  id,
  min_lat, max_lat,
  min_long, max_long,
);

CREATE TABLE tags
(
  tag_id INTEGER PRIMARY KEY,
  tag_name TEXT NOT NULL UNIQUE,
  tag_kind INTEGER NOT NULL,
  tag_rating INTEGER,
  tag_censor INTEGER NOT NULL
);

create INDEX tags_by_name
  ON tags(tag_name);

CREATE TABLE object_tags
(
  tag_id INTEGER NOT NULL,
  obj_id INTEGER NOT NULL
);

create INDEX object_tags_by_object
  ON object_tags(obj_id, tag_id);

create INDEX object_tags_by_tag
  ON object_tags(tag_id, obj_id);

CREATE VIRTUAL TABLE tags_fts USING fts5(
  tag_id,
  tag_name,
  tokenize = 'porter unicode61',
  prefix = 3,
  content = tags,
  content_rowid = tag_id
);
