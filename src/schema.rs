table! {
    answers (id) {
        id -> Integer,
        user_id -> Integer,
        created -> Timestamp,
        option_id -> Integer,
    }
}

table! {
    options (id) {
        id -> Integer,
        data -> Varchar,
        user_id -> Integer,
        question_id -> Integer,
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

table! {
    users (id) {
        id -> Integer,
        name -> Varchar,
        created -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    answers,
    options,
    presentations,
    questions,
    users,
);
