table! {
    documents (id) {
        id -> Text,
        content -> Text,
        lang -> Text,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Text,
        admin -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    documents,
    users,
);
