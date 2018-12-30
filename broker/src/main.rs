extern crate carrier;
extern crate env_logger;
extern crate failure;
extern crate futures;
extern crate rand;
#[macro_use]
extern crate log;
extern crate tokio;
extern crate futurize;
#[macro_use]
extern crate futurize_derive;
extern crate gcmap;
#[macro_use]
extern crate lazy_static;
extern crate dotenv;

use carrier::*;
use failure::Error;
use futures::{Future, Stream};
use std::env;
use dotenv::dotenv;
use std::collections::HashSet;

mod ptrmap;
mod shadow;
mod listener;
mod xlog;

pub fn main() {
    dotenv().ok();

    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "carrier=info,carrier-broker");
    }
    env_logger::init();


    let coordinators : HashSet<identity::Identity> = env::var("COORDINATOR_IDENTITIES")
        .expect("must set env COORDINATOR_IDENTITIES")
        .split(":")
        .map(|v|v.parse().expect("parsing COORDINATOR_IDENTITIES"))
        .collect();

    let secrets = keystore::Secrets::load().unwrap();
    tokio::run(futures::lazy(move || {
        broker(secrets.identity, coordinators).map_err(|e| error!("{}", e))
    }));
}

pub fn broker(secret: identity::Secret, coordinators: HashSet<identity::Identity>) -> impl Future<Item = (), Error = Error> {
    let (lst, sb) = listener::listen(secret.clone()).unwrap();
    let ep = lst.handle();
    lst.for_each(move |ch| {
        let coordinators = coordinators.clone();
        info!("incomming channel {} {}", ch.identity(), ch.addr());
        let addr = ch.addr().clone();
        let sb = sb.clone();
        let ep = ep.clone();
        let ft = ch
            .accept(secret.clone())
            .and_then(move |ch| {
                info!("accepted channel {} for route {}", ch.identity(), ch.route());
                sb.dispatch(ep, ch, addr, coordinators)
            }).and_then(|_| {
                info!("dispatch ended");
                Ok(())
            }).map_err(|e| error!("{}", e));
        tokio::spawn(ft);
        Ok(())
    }).and_then(move |_| Ok(()))
}
