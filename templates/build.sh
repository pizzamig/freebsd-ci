#!/bin/sh
# template variables:
# language: string : {{ language }}
#	the language of the build, like rust
# language_variant: string : {{ language_variant }}
#	the language variant, like stable
# os_family: string : {{ os_family }}
#   the os family, like FreeBSD
# os_version: string : {{ os_version }}
#   the os family, like 12.0
# user: string : {{ user }}
#   the github user name
# project: string : {{ project }}
#   the github project name
# update: boolean : {{ update }}
# 	if the update has to be performed
# upload: boolean : {{ update }}
# 	if the upload has to be performed
# token: string : {{ token }}
#   the github authorization token (valid only if upload is true)
# release_id : u64 : {{ release_id }}
#   the github release to upload the asset to


export HOME=/root
export PATH=/root/.cargo/bin:/sbin:/bin:/usr/sbin:/usr/bin:/usr/local/sbin:/usr/local/bin

if {{ update }} ; then
	rustup update
	pkg upgrade -y
fi

cd /mnt

if ! cargo clippy --release ; then
	exit 1
fi
if ! cargo build --release ; then
	exit 1
fi
if ! cargo test --release ; then
	exit 1
fi


if {{ upload }} ; then
	cargo install --path . -f
	gzip_filename="{{ os_family }}-{{ os_version }}-{{ project }}"
	mv $HOME/.cargo/bin/{{ project }} $gzip_filename
	gzip -S gz
	curl -H "Authorization: bearer {{ token }}"\
		-H "Content-Type: application/gzip" \
		-X POST \
		--data-binary @${gzip_filename}.gz \
		https://uploads.github.com/repos/{{ user }}/{{ project }}/releases/{{ release_id }}/assets\?name\=${gzip_filename}.gz
fi
exit 0
