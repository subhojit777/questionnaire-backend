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
    presentations (id) {
        id -> Integer,
        title -> Varchar,
        user_id -> Integer,
        created -> Timestamp,
    }
}

table! {
    questions (id) {
        id -> Integer,
        title -> Varchar,
        created -> Timestamp,
        presentation_id -> Integer,
        user_id -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    answers,
    presentations,
    questions,
);
