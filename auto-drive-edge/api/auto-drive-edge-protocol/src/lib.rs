pub mod auto_drive_edge_service {
    pub mod v1 {
        tonic::include_proto!("auto_drive_edge_service.v1");
    }
}

pub mod auto_script_lifecycle;
pub mod auto_script_metadata;
pub mod auto_script_permission_level;
pub mod auto_script_simplified_lifecycle;
pub mod log_level;
pub mod log_message;
pub mod log_system_origin;
