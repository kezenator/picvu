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
    objects_fts (id) {
        id -> BigInt,
        title -> Nullable<Text>,
        notes -> Nullable<Text>,
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
    tags_fts (tag_id) {
        tag_id -> BigInt,
        tag_name -> Text,
    }
}

joinable!(attachments_metadata -> objects (obj_id));
allow_tables_to_appear_in_same_query!(objects, attachments_metadata);

joinable!(objects_location -> objects (id));
allow_tables_to_appear_in_same_query!(objects, objects_location);

joinable!(objects_fts -> objects (id));
allow_tables_to_appear_in_same_query!(objects, objects_fts);

joinable!(object_tags -> objects (obj_id));
allow_tables_to_appear_in_same_query!(objects, object_tags);

joinable!(tags_fts -> tags (tag_id));
allow_tables_to_appear_in_same_query!(tags, tags_fts);
