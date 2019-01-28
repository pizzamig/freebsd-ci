# freebsd-ci

This is a tool to implement a local CI/CD server for FreeBSD.  
You'll need a FreeBSD system (a VM is fine) and a Github account
## Installation

You can install `freebsd-ci` on a FreeBSD machine with a rust toolchain installed and a ZFS pool; to use `freebsd-ci`, you'll need root access.

**NOTE**: the FreeBSD machine has to be the same verison of more recent than the container you want to use.  
A FreeBSD 12.0 system can run a FreeBSD 11.2 container. A FreeBSD 11.2 system can not run a FreeBSD 12.0 container.

You can install rust via rustup (https://rustup.rs) or installing the package via `pkg`

FreeBSD 12.0 and the `latest` pkg repository are highly suggested.

The `freebsd-ci` tool is based on `pot` containers. So to install it, you can run:

```console
# pkg install pot
# vi /usr/local/etc/pot/pot.conf # check the configuration
# pot init
```

### Create your images catalog

The most tricky part is to provide an images catalog, that will be used to build software projects.  
In the `pot-images` folder, there are some scripts, ready to use.  
To use them, you can copy them in the pot flavor folder:
```console
# cp pot-images/bsd-ci-rust-* /usr/local/etc/pot/flavours/
# pot ls -F
flavour: bsd-ci-rust-nightly
flavour: bsd-ci-rust-stable
flavour: dns
```

To create the rust stable image for FreeBSD 11.2, use the command:
```console
# pot create -p FreeBSD-11_2-rust-stable -b 11.2 -t single -f bsd-ci-rust-stable
```

To create the rust nightly image for FreeBSD 12.0, use the command:
```console
# pot create -p FreeBSD-12_0-rust-nightly -b 12.0 -t single -f bsd-ci-rust-nightly
```

You can create all the combinations you need.  
Be aware that the name of the pot (the `-p` option) has a fixed form, in order to be found by the freebsd-ci tool.  
You can list your `pot`, via the command:
```console
# pot ls
```

**NOTE**: `pot` names cannot contains dots; for this reason, the dot is subsituted by an underscore.
### The freebsd-ci.conf

The `freebsd-ci.conf` file is needed to store your github token. It's a simple toml file:
```toml
[tokens]
github = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

Github tokens can be obtained at the url https://github.com/settings/tokens/new and you need the `repo` and the `user` scope.


### The build.sh template





