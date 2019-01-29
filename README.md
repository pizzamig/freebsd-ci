# freebsd-ci

This is a tool to implement a local CI/CD server for FreeBSD.  
You'll need a FreeBSD system (a VM is fine) and a Github account
## Installation

You can install `freebsd-ci` on a FreeBSD machine with a rust toolchain installed and a ZFS pool; to use `freebsd-ci`, you'll need root access.

**NOTE**: the FreeBSD machine has to be the same verison of more recent than the container you want to use.  
A FreeBSD 12.0 system can run a FreeBSD 11.2 container. A FreeBSD 11.2 system can not run a FreeBSD 12.0 container.

You can install rust via rustup (https://rustup.rs) or installing the package via `pkg`

To install `freebsd-ci` you can clone this repo in your directory and then run:
```console
# cargo install --path .
```

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


## How to use it

The command line tools is called `freebsd-ci` and how to use it can be invoked in this way:

```console
# freebsd-ci --help
USAGE:
    freebsd-ci [FLAGS] [OPTIONS] --project <project_name> --user-name <user_name>

FLAGS:
    -f, --force                A Flag to force operations (i.e. remove fscomp or images with the same name)
    -h, --help                 Prints help information
    -B, --build-script-only    A Flag to rendert the build script only (on stdout)
    -v, --verbose              Enable the verbose output No multiple occurrences are supported
    -V, --version              Prints version information

OPTIONS:
    -b, --build <build_template>    The pathname to the build-sh template [default: ./templates/build.sh]
    -c, --config <configfile>       The pathname to the toml configuration file [default: ./freebsd-ci.conf]
    -P, --project <project_name>    Github project name
    -T, --tag-name <tag_name>       Tag name: Using this option, a tag will be built. If a related release is found, the
                                    artifacts will be uploaded
    -U, --user-name <user_name>     Github user name
```
where `username` is the github username and `project-name` is the github project name and are manddatory.

To test that you installation works, from the project directory, you can try to build my test project:
```
# freebsd-ci -U pizzamig -P ci-test
```

### The build.sh template
The build script template can be customized. in `templates/build.sh` there is a standard script with all template variables listed and documented.  
If you want to test your script template you can use the `-b` option to point to your custom template and the flag -B that will show the output at the console, without executing the build (the project will be still downloaded to read the YAML file)

### The deploy to github

If the tool is invoked with the `-T` option, then the a tarball can be built and uploaded to github to the relative release.  
The upload is performed by the build script and it can be disabled in the YAML file

### The YAML file

The YAML file has to be stored in the root directory with the name `.bsd-ci.yml`.  
Here a commented example:
```yaml
os: FreeBSD		# the operating system

FreeBSD:		# the version of the operating system
    - '11.2'
    - '12.0'

update: true	# perform the toolchain/pkg upgrade before the build

language: rust	# the project language

rust:			# the language variant
    - stable
    - beta
    - nightly

no_deploy:		# in case of a tag (deploy) which version to exclude
    rust:
        - nightly
        - beta
```




