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
        created_offset -> Integer,
        modified_timestamp -> BigInt,
        modified_offset -> Integer,
        activity_timestamp -> BigInt,
        activity_offset -> Integer,
        title -> Nullable<Text>,
        notes -> Nullable<Text>,
        rating -> Nullable<Integer>,
        censor -> Integer,
        latitude -> Nullable<Double>,
        longitude -> Nullable<Double>,
    }
}

table! {
    attachments_metadata (obj_id) {
        obj_id -> BigInt,
        filename -> Text,
        created_timestamp -> BigInt,
        created_offset -> Integer,
        modified_timestamp -> BigInt,
        modified_offset -> Integer,
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

joinable!(attachments_metadata -> objects (obj_id));
allow_tables_to_appear_in_same_query!(objects, attachments_metadata);

joinable!(objects_location -> objects (id));
allow_tables_to_appear_in_same_query!(objects, objects_location);

joinable!(objects_fts -> objects (id));
allow_tables_to_appear_in_same_query!(objects, objects_fts);
