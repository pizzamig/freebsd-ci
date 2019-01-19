#!/bin/sh

[ -w /etc/pkg/FreeBSD.conf ] && sed -i '' 's/quarterly/latest/' /etc/pkg/FreeBSD.conf
ASSUME_ALWAYS_YES=yes pkg bootstrap
touch /etc/rc.conf
sysrc sendmail_enable="NONE"

pkg install -y ca_root_nss curl

fetch -o /root/rustup.sh https://sh.rustup.rs
sh /root/rustup.sh -y --default-toolchain stable
export PATH="$HOME/.cargo/bin:$PATH"
echo setenv PATH $HOME/.cargo/bin:'$PATH' >> $HOME/.cshrc

rustup component add clippy-preview
rustup component add rustfmt

pkg clean -aqy
