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
        title -> Nullable<Text>,
    }
}

table! {
    attachments_metadata (id) {
        id -> Text,
        filename -> Text,
        created -> BigInt,
        modified -> BigInt,
        mime -> Text,
        size -> BigInt,
        hash -> Text,
    }
}

table! {
    attachments_data (id) {
        id -> Text,
        bytes -> Blob,
    }
}
