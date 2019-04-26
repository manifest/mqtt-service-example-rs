#![feature(await_macro, async_await, futures_api)]

use futures::executor;

fn main() {
    env_logger::init();
    executor::block_on(app::run()).expect("Error running an executor");
}

mod app;
