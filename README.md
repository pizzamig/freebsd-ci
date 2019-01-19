# freebsd-ci

This is a tool to implement a local CI/CD server for FreeBSD.
You'll need a FreeBSD system (a VM is fine) and a Github account

## Installation

You can install freebsd-ci on a FreeBSD machine where you have root access.
You can install rust via rustup (rustup.rs) 
FreeBSD 12.0 and the `latest` pkg repository are highly suggested.

The tool is based on `pot` containers. So to install it run:

```console
# pkg install pot
# pot init
```

### Create your images catalog

The most tricky part is to provide an images catalog, that will be used to build software projects.
In the `pot-images` folder, there are some scripts, ready to use.
To use them, you can copy the in the pot flavor folder:
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

You can create all the combinations you need. Be aware that the name of the pot (the `-p` option) has a fixed form.
You can list your `pot`, via the command:
```console
# pot ls
```

### The freebsd-ci.conf

The `freebsd-ci.conf` file is needed to store your github token. It's a simple toml file:
```toml
[tokens]
github = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

Github tokens can be obtained at the url https://github.com/settings/tokens/new and you need the `repo` and the `user` scope.


### The build.sh template





