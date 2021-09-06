#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

pub mod apikey;
pub mod cmd;
pub mod db;
pub mod error;
pub mod games;
pub mod models;
pub mod schema;
pub mod server;
pub mod tournament;
