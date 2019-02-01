# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] 2019-02-01
### Added
- github support
- build.sh: using template support via tera
- build.sh: add template context
- add a --build-script-only flag, to render the build.sh script
- add a --tag-name option, to support automatic asset upload
- add support to custom build template scripts
- add support to the no_deploy clause in the yaml file
- asset support: delete the asset, if already present, allowing "update"
