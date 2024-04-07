pub mod command_server {
    include!(concat!(env!("OUT_DIR"), "/command_server.rs"));
}

pub mod failure_details {
    include!(concat!(env!("OUT_DIR"), "/failure_details.rs"));
}