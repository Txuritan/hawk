use uuid::Uuid;

#[derive(serde::Deserialize)]
pub(crate) struct Login {
    pub username: String,
    pub password: String,
    pub email: String,
}

pub(crate) struct Video {
    pub id: Uuid,
    pub ext: String,
}
