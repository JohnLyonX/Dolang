mod env;
mod fs;
mod http_client;
mod json;
mod math;
mod str;
mod time;
mod uuid;

use super::RuntimeContext;

pub fn register_stdlib_native_modules(context: &mut RuntimeContext) {
    fs::register(context);
    env::register(context);
    http_client::register(context);
    str::register(context);
    math::register(context);
    json::register(context);
    time::register(context);
    uuid::register(context);
}
