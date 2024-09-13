#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

pub mod operation;
pub mod persistent_data;
mod persistent_user_data;
mod session_data;
mod shared_data;

#[derive(Debug, Clone, Default)]
pub struct ModelingModule {}

impl module::Module for ModelingModule {
    type PersistentData = persistent_data::PersistentData;
    type PersistentUserData = persistent_user_data::PersistentUserData;
    type SessionData = session_data::SessionData;
    type SharedData = shared_data::SharedData;

    fn name() -> String {
        "Modeling".to_string()
    }

    fn uuid() -> uuid::Uuid {
        uuid::Uuid::parse_str("04d338d9-b7a9-4f5a-b04d-724466f4058f").expect("static UUID")
    }
}
