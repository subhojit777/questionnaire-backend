use actix::{Handler, Message};
use diesel::query_dsl::RunQueryDsl;
use diesel::MysqlConnection;
use models::NewQuestion;
use DbExecutor;

impl Message for NewQuestion {
    type Result = Result<(), super::error::Db>;
}

impl Handler<NewQuestion> for DbExecutor {
    type Result = Result<(), super::error::Db>;

    fn handle(&mut self, msg: NewQuestion, _ctx: &mut Self::Context) -> Self::Result {
        use schema::questions::dsl::questions;
        let connection: &MysqlConnection = &self.0.get().unwrap();

        diesel::insert_into(questions)
            .values(&msg)
            .execute(connection)
            .expect("Error saving the question");

        Ok(())
    }
}
