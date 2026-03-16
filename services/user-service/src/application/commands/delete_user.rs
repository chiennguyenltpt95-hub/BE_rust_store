use uuid::Uuid;

#[derive(Debug)]
pub struct DeleteUserCommand {
    pub user_id: Uuid,
}
