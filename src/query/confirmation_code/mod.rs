pub mod verify;
pub mod get_user_id;

pub use verify::verify_confirmation_code;
pub use get_user_id::get_user_id_from_registration_code;