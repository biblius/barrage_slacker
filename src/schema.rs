table! {
    messages (id) {
        id -> Int4,
        sender -> Varchar,
        body -> Text,
        time_sent -> Timestamp,
    }
}
