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
        created_timestring -> Text,
        modified_timestamp -> BigInt,
        modified_timestring -> Text,
        activity_timestamp -> BigInt,
        activity_timestring -> Text,
        obj_type -> Text,
        title -> Nullable<Text>,
        notes -> Nullable<Text>,
        latitude -> Nullable<Double>,
        longitude -> Nullable<Double>,
    }
}

table! {
    attachments_metadata (obj_id) {
        obj_id -> Text,
        filename -> Text,
        created -> BigInt,
        modified -> BigInt,
        mime -> Text,
        size -> BigInt,
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
