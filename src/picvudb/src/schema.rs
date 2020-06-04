table! {
    db_properties (name) {
        name -> Text,
        value -> Text,
    }
}

table! {
    objects (id) {
        id -> Text,
        created_timestamp -> BigInt,
        created_offset -> Integer,
        modified_timestamp -> BigInt,
        modified_offset -> Integer,
        activity_timestamp -> BigInt,
        activity_offset -> Integer,
        obj_type -> Text,
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
        obj_id -> Text,
        filename -> Text,
        created_timestamp -> BigInt,
        created_offset -> Integer,
        modified_timestamp -> BigInt,
        modified_offset -> Integer,
        mime -> Text,
        size -> BigInt,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        duration -> Nullable<Integer>,
        hash -> Text,
    }
}

table! {
    attachments_data (obj_id, offset) {
        obj_id -> Text,
        offset -> BigInt,
        bytes -> Blob,
    }
}

joinable!(attachments_metadata -> objects (obj_id));
allow_tables_to_appear_in_same_query!(objects, attachments_metadata);
