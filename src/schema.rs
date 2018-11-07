table! {
    answers (id) {
        id -> Integer,
        question_id -> Integer,
        title -> Varchar,
        user_id -> Integer,
        created -> Timestamp,
    }
}

table! {
    question (id) {
        id -> Integer,
        title -> Varchar,
        created -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    answers,
    question,
);
