mod env;
mod fs;
mod json;
mod math;
mod str;
mod time;

use super::RuntimeContext;

pub fn register_stdlib_native_modules(context: &mut RuntimeContext) {
    fs::register(context);
    env::register(context);
    str::register(context);
    math::register(context);
    json::register(context);
    time::register(context);
}
