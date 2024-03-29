//! This module is the root of the client-server API.
//!
//! reference: https://spec.matrix.org/unstable/client-server-api

pub mod login;
pub mod logout;
pub mod register;
pub mod create_room;

pub mod account;
pub mod profile;
pub mod uiaa;
