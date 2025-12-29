use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub id: u64,
    pub name: String,
    pub channel_type: i32,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub id: u64,
    pub name: String,
    pub color: u32,
    pub position: i32,
    pub mentionable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub permissions: u64,
    pub is_admin: bool,
    pub is_owner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSelfRoleRequest {
    pub user_id: u64,
    pub title: String,
    pub body: String,
    pub selection_type: String,
    pub channel_id: String,
    pub roles: Vec<SelfRoleData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfRoleData {
    pub role_id: String,
    pub emoji: String,
}
