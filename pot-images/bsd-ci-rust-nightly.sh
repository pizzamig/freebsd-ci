#!/bin/sh

if [ "$(uname -r)" != "10.1-RELEASE" ]; then
	[ -w /etc/pkg/FreeBSD.conf ] && sed -i '' 's/quarterly/latest/' /etc/pkg/FreeBSD.conf
	ASSUME_ALWAYS_YES=yes pkg bootstrap
fi
touch /etc/rc.conf
sysrc sendmail_enable="NONE"

pkg install -y ca_root_nss curl
fetch -o /root/rustup.sh https://sh.rustup.rs
sh /root/rustup.sh -y --default-toolchain nightly
export PATH="$HOME/.cargo/bin:$PATH"
rustup component add clippy-preview
rustup component add rustfmt
echo setenv PATH $HOME/.cargo/bin:'$PATH' >> $HOME/.cshrc
