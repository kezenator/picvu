table! {
    db_properties (name) {
        name -> Text,
        value -> Text,
    }
}

table! {
    objects (id) {
        id -> BigInt,
        created_timestamp -> BigInt,
        created_offset -> Nullable<Integer>,
        modified_timestamp -> BigInt,
        modified_offset -> Nullable<Integer>,
        activity_timestamp -> BigInt,
        activity_offset -> Nullable<Integer>,
        title -> Nullable<Text>,
        notes -> Nullable<Text>,
        rating -> Nullable<Integer>,
        censor -> Integer,
        location_source -> Nullable<Integer>,
        latitude -> Nullable<Double>,
        longitude -> Nullable<Double>,
        altitude -> Nullable<Double>,
        tag_set -> Nullable<Text>,
        ext_ref_type -> Nullable<Text>,
        ext_ref_id -> Nullable<Text>,
    }
}

table! {
    attachments_metadata (obj_id) {
        obj_id -> BigInt,
        filename -> Text,
        created_timestamp -> BigInt,
        created_offset -> Nullable<Integer>,
        modified_timestamp -> BigInt,
        modified_offset -> Nullable<Integer>,
        mime -> Text,
        size -> BigInt,
        orientation -> Nullable<Integer>,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        duration -> Nullable<Integer>,
        hash -> Text,
    }
}

table! {
    attachments_data (obj_id, offset) {
        obj_id -> BigInt,
        offset -> BigInt,
        bytes -> Blob,
    }
}

table! {
    #[sql_name = "objects_fts"]
    objects_fts_insert (rowid) {
        rowid -> BigInt,
        title -> Nullable<Text>,
        notes -> Nullable<Text>,
    }
}

table! {
    #[sql_name = "objects_fts"]
    objects_fts_query (rowid) {
        rowid -> BigInt,

        #[sql_name = "objects_fts"]
        whole_row -> Text,
        rank -> Float,
    }
}

table! {
    objects_location (id) {
        id -> BigInt,
        min_lat -> Double,
        max_lat -> Double,
        min_long -> Double,
        max_long -> Double,
    }
}

table! {
    tags (tag_id) {
        tag_id -> BigInt,
        tag_name -> Text,
        tag_kind -> Integer,
        tag_rating -> Nullable<Integer>,
        tag_censor -> Integer,
    }
}

table! {
    object_tags (tag_id, obj_id) {
        tag_id -> BigInt,
        obj_id -> BigInt,
    }
}

table! {
    #[sql_name = "tags_fts"]
    tags_fts_insert (rowid) {
        rowid -> BigInt,
        tag_name -> Text,
    }
}

table! {
    #[sql_name = "tags_fts"]
    tags_fts_query (rowid) {
        rowid -> BigInt,

        #[sql_name = "tags_fts"]
        whole_row -> Text,
        rank -> Float,
    }
}

joinable!(attachments_metadata -> objects (obj_id));
allow_tables_to_appear_in_same_query!(objects, attachments_metadata);

joinable!(objects_location -> objects (id));
allow_tables_to_appear_in_same_query!(objects, objects_location);

joinable!(objects_fts_query -> objects (rowid));
allow_tables_to_appear_in_same_query!(objects, objects_fts_query);

joinable!(object_tags -> objects (obj_id));
allow_tables_to_appear_in_same_query!(objects, object_tags);

joinable!(tags_fts_query -> tags (rowid));
allow_tables_to_appear_in_same_query!(tags, tags_fts_query);
