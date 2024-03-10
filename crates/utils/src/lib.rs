//! Various utilities used throughout the project.
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(dead_code)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

pub mod transaction;

pub use self::transaction::Transaction;
