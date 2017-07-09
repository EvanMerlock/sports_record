use std::collections::HashMap;

use unsafe_code::{Packet};
use framework::{VideoResult};

trait VideoSupplier {

    fn initialize_video_supplier(map: HashMap<String, String>) -> VideoResult<()>;
    fn get_packet() -> VideoResult<Packet>;
    fn close_video_supplier() -> VideoResult<()>;

}
