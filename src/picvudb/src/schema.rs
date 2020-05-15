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
        label -> Text,
    }
}
