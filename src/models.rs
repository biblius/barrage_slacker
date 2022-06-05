#[derive(Queryable)]
pub struct Message {
    pub id: i32,
    pub sender: String,
    pub body: String,
    pub time_sent: chrono::NaiveDateTime
}