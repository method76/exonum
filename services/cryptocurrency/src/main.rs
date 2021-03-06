extern crate exonum;
extern crate exonum_configuration;

use exonum::helpers::{self, fabric::NodeBuilder};
use exonum_configuration as configuration;
use exonum_cryptocurrency_advanced as cryptocurrency;

fn main() {
    exonum::crypto::init();
    helpers::init_logger().unwrap();

    let node = NodeBuilder::new()
        .with_service(Box::new(configuration::ServiceFactory))
        .with_service(Box::new(cryptocurrency::ServiceFactory));
    node.run();
}
