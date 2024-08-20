// @generated automatically by Diesel CLI.

diesel::table! {
    Applications (id) {
        id -> Int4,
        jobId -> Int4,
        #[max_length = 255]
        applicant -> Varchar,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

diesel::table! {
    Jobs (id) {
        id -> Int4,
        #[max_length = 255]
        title -> Varchar,
        description -> Text,
        #[max_length = 255]
        salary -> Varchar,
        #[max_length = 255]
        employer -> Varchar,
        createdAt -> Timestamptz,
        updatedAt -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    Applications,
    Jobs,
    users,
);
