table! {
    answers (id) {
        id -> Integer,
        uuid -> Varchar,
        question_id -> Integer,
        title -> Varchar,
        user_id -> Integer,
        created -> Integer,
    }
}

table! {
    question (id) {
        id -> Integer,
        uuid -> Varchar,
        title -> Varchar,
        created -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    answers,
    question,
);
