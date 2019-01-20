#!/bin/sh

export HOME=/root
export PATH=/root/.cargo/bin:/sbin:/bin:/usr/sbin:/usr/bin:/usr/local/sbin:/usr/local/bin

cd /mnt

if ! cargo clippy --release ; then
	exit 1
fi
if ! cargo build --release ; then
	exit 1
fi
if !  cargo test --release ; then
	exit 1
fi
exit 0
