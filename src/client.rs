use ambient_api::prelude::*;
#[main]
pub fn main() {

    ambient_api::messages::Frame::subscribe(move |_| {
        let (_delta, input) = input::get_delta();

        let mut displace = Vec2::ZERO;
        if input.keys.contains(&KeyCode::W) {
            displace.y -= 1.0;
        }
        if input.keys.contains(&KeyCode::S) {
            displace.y += 1.0;
        }
        if input.keys.contains(&KeyCode::A) {
            displace.x -= 1.0;
        }
        if input.keys.contains(&KeyCode::D) {
            displace.x += 1.0;
        }

        messages::Input::new(displace, input.mouse_delta).send_server_unreliable();
    });
}
