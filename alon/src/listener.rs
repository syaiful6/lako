use std::net::ToSocketAddrs;

use tokio::net::TcpListener;
use tokio::runtime::{self, Runtime};

// create a new tokio Runtime
fn new_runtime(threads: usize) -> Runtime {
	runtime::Builder::new()
		.core_threads(threads)
		.new_prefix("lako-worker-")
		.build()
		.unwrap()
}

fn tcp_listener<A>(addr: A) -> TcpListener
where
	A: ToSocketAddrs + 'static,
{
	let addr = addr
		.to_socket_addrs()
		.expect("unable to parse listener address")
		.next()
		.expect("unable to resolve listener address");

	TcpListener::bind(&addr).expect("unable to open TCP Listener")
}