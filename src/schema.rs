// @generated automatically by Diesel CLI.

diesel::table! {
    blobs (hash) {
        hash -> Text,
        content -> Binary,
    }
}
