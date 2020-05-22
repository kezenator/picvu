table! {
    db_properties (name) {
        name -> Text,
        value -> Text,
    }
}

table! {
    objects (id) {
        id -> Text,
        added_timestamp -> BigInt,
        added_timestring -> Text,
        changed_timestamp -> BigInt,
        changed_timestring -> Text,
        obj_type -> Text,
        title -> Nullable<Text>,
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
