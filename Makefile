# holochain-rust Makefile
# o Build Holochain targets, either via Nix, or using the local toolchain

.PHONY: all build install help
all: build

build: build_holochain build_cli build_nodejs

install: build install_cli install_conductor

help:
	@echo "run 'make all' to build all the libraries and binaries, and the nodejs bin-package"
	@echo "run 'make nix-all' to build all the libraries and binaries, and the nodejs bin-package using a Nix environment"
	@echo "run 'make install' to build and install all the libraries and binaries, and the nodejs bin-package"
	@echo "run 'make test' to execute all the tests"
	@echo "run 'make test_app_spec' to build and test app_spec API tests"
	@echo "run 'make clean' to clean up the build environment"
	@echo "run 'make test_holochain' to test holochain builds"
	@echo "run 'make test_cli' to build and test the command line tool builds"
	@echo "run 'make install_cli' to build and install the `hc` command-line tool"
	@echo "run 'make install_conductor' to build and install the `holochain` command-line tool"
	@echo "run 'make test-something' to run cargo tests matching 'something'"

# NIX Shell build/install/test: See `shell.nix` and the Nix environment manager `nix-shell`.  We
# don't want to contaminate the Nix build environment with the environment variables required to get
# machine-local builds to work (eg. RUST_SODIUM_...), so use --pure.  To install Nix:
# https://nixos.org/nix/download.html
.PHONY: nix-all nix-build nix-install nix-test
nix-all: nix-build nix-test

nix-build:
	nix-shell --pure --run hc-build-wasm

nix-install: nix-build
	nix-shell --pure --run hc-install-cli			# hc
	nix-shell --pure --run hc-install-conductor		# holochain

nix-test:
	rm -rf nodejs_conductor/build
	nix-shell --pure --run hc-install-node-conductor	# nodejs_conductor
	nix-shell --pure --run hc-test-app-spec

# We use bash features in some of our Makefile shell scripting
SHELL = /bin/bash

# The Rust versions required for Holochain development, and the required cargo options are configured here:
CORE_RUST_VERSION ?= nightly-2019-01-24
TOOLS_RUST_VERSION ?= nightly-2019-01-24
CARGO = RUSTFLAGS="-Z external-macro-backtrace -D warnings" rustup run $(CORE_RUST_VERSION) cargo -Z config-profile $(CARGO_ARGS)
CARGO_TOOLS = RUSTFLAGS="-Z external-macro-backtrace -D warnings" rustup run $(TOOLS_RUST_VERSION) cargo -Z config-profile $(CARGO_ARGS)
CARGO_TARPULIN_INSTALL = RUSTFLAGS="--cfg procmacro2_semver_exempt -D warnings" cargo -Z config-profile $(CARGO_ARGS) +$(CORE_RUST_VERSION)

# All rustup and cargo invocations executed (directly in this Makefile, or indirectly eg. via npm)
# must include these environment variables.  If we'd like to see Rust back-traces:
export RUST_BACKTRACE=1

# There are 3 methods to obtain the libsodium encryption library required by holochain-rust,
# selectable by setting/clearing various RUST_SODIUM_...  environment variables (see:
# https://github.com/maidsafe/rust_sodium).  These selections are implemented and enforced in
# rust_sodium-sys/build.rs.  The default is to download/compile libsodium 1.0.17 (06-Jan-2019).

# 0) DEFAULT: Downloaded/compiled by holochain-rust build: select by clearing RUST_SODIUM_LIB_DIR
# and RUST_SODIUM_USE_PKG_CONFIG.  Some systems require libsodium to be configured and built with
# `--disable-pie`; select this here by setting RUST_SODIUM_DISABLE_PIE.
#export RUST_SODIUM_DISABLE_PIE=1

# 1) System installed: select by setting RUST_SODIUM_LIB_DIR. We need to find the location of the
# system's libsodium dynamic library: at least version 1.0.12 is required.  On Mac, `brew install
# libsodium`.  On Ubuntu Bionic (libsodium 1.0.16), Consmic (libsodium 1.0.16) and Disco (libsodium
# 1.0.17): `apt-get install libsodium-dev`.  On Debian Stretch (stable, libsodium 1.0.11 *to old*,
# add `buster` to /etc/apt/sources...), or Debian Buster (testing, libsodium 1.0.17): `apt-get -t
# buster -u install libsodium-dev`.  On other Linux distros, ensure at least libsodium 1.0.12+ is
# installed.
RUST_SODIUM_LIB=$(shell find /usr/local/lib /usr/lib /lib -name 'libsodium.so' -o -name 'libsodium.dylib' 2>/dev/null | head -1)
export RUST_SODIUM_LIB_DIR=$(dir $(RUST_SODIUM_LIB))
export RUST_SODIUM_SHARED=1

# 2) Rust `pkg_config::probe_library`-detected: select by setting RUST_SODIUM_USE_PKG_CONFIG.
#export RUST_SODIUM_USE_PKG_CONFIG=1

# list all the "C" binding tests that have been written
C_BINDING_DIRS = $(sort $(dir $(wildcard c_binding_tests/*/)))

# list all the "C" binding test executables that should be produced
C_BINDING_TESTS = $(foreach dir,$(C_BINDING_DIRS),target/debug/c_binding_tests/$(shell basename $(dir))/test_executable)

# list all the extraneous files that will be generated when running tests
C_BINDING_CLEAN = $(foreach dir,$(C_BINDING_DIRS),$(dir)Makefile $(dir).qmake.stash)

# build artifact / dependency checking is handled by our sub-tools
# we can just try to build everything every time, it should be efficient
.PHONY: lint \
	c_binding_tests ${C_BINDING_DIRS} \
	test ${C_BINDING_TESTS} \
	test_holochain \
	clean ${C_BINDING_CLEAN}

# apply formatting / style guidelines
lint: fmt_check clippy

# Check if Rust version is correct, and prompts to offer to change to the correct version.  Requires
# RUST_VERSION to be set, as appropriate for whatever target is being installed (defaults to
# CORE_RUST_VERSION; see install_rustup..., below).  We'll also export PATH to default location of
# Rust installation for use here in the Makefile, in case this is the first time rustup has been
# installed/run, and we don't have a rustup-modified .profile loaded yet.  If not connected to a
# terminal (stdin is a tty), or running under a Continuous Integration test (CI), defaults to
# automatically installing and changing the default Rust version (under the assumption that the
# invoker of the Makefile target knows what they want, under headless automated procedures like
# CI). Otherwise, entering "no<return>" rejects installing/changing the Rust version (and we assume
# you know what you're doing, eg. testing some new Rust toolchain version that you've installed)
export PATH := $(HOME)/.cargo/bin:$(PATH)
RUST_VERSION = $(CORE_RUST_VERSION)
.PHONY: version_rustup

version_rustup:
	@if which rustup >/dev/null; then \
	    echo -e "\033[0;93m## Current Rust version installed (need: '$(RUST_VERSION)'): ##\033[0m"; \
	    if ! rustup override list 2>/dev/null | grep "^$(PWD)\s*$(RUST_VERSION)"; then \
		rustup show; rustup override list; \
		echo -e "\033[0;93m## Change $(PWD) Rust version override to '$(RUST_VERSION)' ##\033[0m"; \
		[ -t 1 ] && [ -t 0 ] && [[ "$(CI)" == "" ]] && read -p "Continue? (Y/n) " yes; \
		if [[ "$${yes:0:1}" != "n" ]] && [[ "$${yes:0:1}" != "N" ]]; then \
		    echo -e "\033[0;93m## Selecting Rust version '$(RUST_VERSION)'... ##\033[0m"; \
		    rustup override set $(RUST_VERSION); \
		fi; \
	    fi; \
	fi


# Actual installation of Rust $(RUST_VERSION) via curl
.PHONY: curl_rustup
curl_rustup:
	@if ! which rustup >/dev/null; then \
	    echo -e "\033[0;93m## Installing Rust $(RUST_VERSION)... ##\033[0m"; \
	    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain $(RUST_VERSION) -y; \
	fi

# idempotent install rustup with the default toolchain set for Holochain core
# best for green fields Rust installation
.PHONY: install_rustup
install_rustup:		RUST_VERSION = $(CORE_RUST_VERSION)
install_rustup: version_rustup curl_rustup

# idempotent install rustup with the default toolchain set for tooling
# best for CI based on tools only.
.PHONY: install_rustup_tools
install_rustup_tools:	RUST_VERSION = $(TOOLS_RUST_VERSION)
install_rustup_tools: version_rustup curl_rustup

# idempotent installation of core toolchain.  Changes default toolchain to CORE_RUST_VERSION.
.PHONY: core_toolchain
core_toolchain: RUST_VERSION=$(CORE_RUST_VERSION)
core_toolchain: version_rustup install_rustup

# idempotent installation of tools toolchain.  Changes default toolchain to TOOLS_RUST_VERSION.
.PHONY: tools_toolchain
tools_toolchain: RUST_VERSION=$(TOOLS_RUST_VERSION)
tools_toolchain: version_rustup install_rustup_tools

# idempotent addition of wasm target in current (default: CORE_RUST_VERSION) toolchain
.PHONY: ensure_wasm_target
ensure_wasm_target: core_toolchain
	rustup target add wasm32-unknown-unknown

# idempotent installation of development tooling; RUST_VERSION defaults to TOOLS_RUST_VERSION
# Since the default toolchain has been changed (see: tools_toolchain, version_rustup), we can
# install the component without specifying which toolchain version to use)
.PHONY: install_rust_tools
install_rust_tools: tools_toolchain
	@if ! rustup component list | grep -q 'rustfmt-preview.*(installed)'; then \
		echo -e "\033[0;93m## Installing rustfmt (rust formatting) tools ##\033[0m"; \
		rustup component add rustfmt-preview; \
	fi
	@if ! rustup component list | grep -q 'clippy-preview.*(installed)'; then \
		echo -e "\033[0;93m## Installing clippy (rust linting) tools ##\033[0m"; \
		rustup component add clippy-preview; \
	fi

# idempotent installation of code coverage CI/testing tools
.PHONY: install_ci
install_ci: core_toolchain
	@if ! $(CARGO) install --list | grep -q 'cargo-tarpaulin'; then \
		echo -e "\033[0;93m## Installing cargo-tarpaulin (code coverage) tools ##\033[0m"; \
		$(CARGO_TARPULIN_INSTALL) install cargo-tarpaulin --force; \
	fi

.PHONY: install_mdbook
install_mdbook: tools_toolchain
	@if ! $(CARGO_TOOLS) install --list | grep -q 'mdbook'; then \
		echo -e "\033[0;93m## Installing mdbook (documentation generation) tools ##\033[0m"; \
		$(CARGO_TOOLS) install mdbook --vers "^0.2.2"; \
	fi

# list all our found "C" binding tests
c_binding_tests: ${C_BINDING_DIRS}

# build all our found "C" binding tests
${C_BINDING_DIRS}:
	qmake -o $@Makefile $@qmake.pro
	cd $@; $(MAKE)

# execute all tests: holochain, command-line tools, app spec, nodejs conductor, and "C" bindings
.PHONY: test
test: test_holochain test_cli test_app_spec c_binding_tests ${C_BINDING_TESTS}

test_holochain: build_holochain
	RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc

# Execute cargo tests matching %
# Eg. make test-stacked will run "cargo test stacked"
test-%: build_holochain
	RUSTFLAGS="-D warnings" $(CARGO) test $* -- --nocapture

test_cli: build_cli
	@echo -e "\033[0;93m## Testing hc command... ##\033[0m"
	cd cli && RUSTFLAGS="-D warnings" $(CARGO) test

test_app_spec: RUST_VERSION=$(CORE_RUST_VERSION)
test_app_spec: version_rustup ensure_wasm_target install_cli build_nodejs_conductor
	@echo -e "\033[0;93m## Testing app_spec... ##\033[0m"
	cd app_spec && ./build_and_test.sh

build_nodejs_conductor: RUST_VERSION=$(CORE_RUST_VERSION)
build_nodejs_conductor: version_rustup core_toolchain
	@echo -e "\033[0;93m## Building nodejs_conductor... ##\033[0m"
	./scripts/build_nodejs_conductor.sh

c_build: core_toolchain
	cd dna_c_binding && $(CARGO) build

test_c_ci: c_build c_binding_tests ${C_BINDING_TESTS}

.PHONY: wasm_build
wasm_build: ensure_wasm_target
	@echo -e "\033[0;93m## Building wasm targets... ##\033[0m"
	cd core/src/nucleus/actions/wasm-test && $(CARGO) build --release --target wasm32-unknown-unknown
	cd conductor_api/wasm-test && $(CARGO) build --release --target wasm32-unknown-unknown
	cd conductor_api/test-bridge-caller && $(CARGO) build --release --target wasm32-unknown-unknown
	cd hdk-rust/wasm-test && $(CARGO) build --release --target wasm32-unknown-unknown
	cd wasm_utils/wasm-test/integration-test && $(CARGO) build --release --target wasm32-unknown-unknown


.PHONY: build_holochain libsodium_version
libsodium_version:
	@[ ! -z "$${RUST_SODIUM_LIB_DIR+defined}" ] \
	    && echo -e "\033[0;93m## Configured rust_sodium-sys -- with system libsodium \"$(RUST_SODIUM_LIB)\" in: $${RUST_SODIUM_LIB_DIR:-(unknown)}) ##\033[0m" \
	    || ( [ ! -z "$${RUST_SODIUM_USE_PKG_CONFIG+defined}" ] \
		&& echo -e "\033[0;93m## Configured rust_sodium-sys -- with pkg_config libsodium ##\033[0m" \
		|| ( echo -e "\033[0;93m## Configured rust_sodium-sys -- with download/build libsodium ##\033[0m" ))

build_holochain: core_toolchain wasm_build libsodium_version
	@echo -e "\033[0;93m## Building holochain... ##\033[0m"
	$(CARGO) build --all --exclude hc

.PHONY: build_cli
build_cli: core_toolchain ensure_wasm_target
	@echo -e "\033[0;93m## Building hc command... ##\033[0m"
	$(CARGO) build -p hc --release

.PHONY: build_conductor
build_conductor: core_toolchain ensure_wasm_target
	@echo -e "\033[0;93m## Building holochain command... ##\033[0m"
	$(CARGO) build -p holochain --release

.PHONY: build_nodejs
build_nodejs:
	@echo -e "\033[0;93m## Building nodejs interface... ##\033[0m"
	cd nodejs_conductor && npm run compile && mkdir -p bin-package && cp native/index.node bin-package

.PHONY: install_cli
install_cli: build_cli
	@echo -e "\033[0;93m## Installing hc command... ##\033[0m"
	$(CARGO) install -f --path cli

.PHONY: install_conductor
install_conductor: build_conductor
	@echo -e "\033[0;93m## Installing holochain command... ##\033[0m"
	$(CARGO) install -f --path conductor

.PHONY: code_coverage
code_coverage: core_toolchain wasm_build install_ci
	$(CARGO) tarpaulin --ignore-tests --timeout 600 --all --out Xml --skip-clean -v -e holochain_core_api_c_binding -e hdk -e hc -e holochain_core_types_derive

.PHONY: code_coverage_crate
code_coverage_crate: core_toolchain wasm_build install_ci
	$(CARGO) tarpaulin --ignore-tests --timeout 600 --skip-clean -v -p $(CRATE)

fmt_check: install_rust_tools
	$(CARGO_TOOLS) fmt -- --check

clippy: install_rust_tools
	$(CARGO_TOOLS) clippy -- -A clippy::needless_return --A clippy::useless_attribute

fmt: install_rust_tools
	$(CARGO_TOOLS) fmt

# execute all the found "C" binding tests
${C_BINDING_TESTS}:
	$@

# clean up the target directory and all extraneous "C" binding test files
clean: ${C_BINDING_CLEAN}
	@for target in $$( find . -type d -a -name 'target' ); do \
	    echo -e "\033[0;93m## Removing $${target} ##\033[0m"; \
	    $(RM) -rf $${target}; \
        done
	@$(RM) -rf nodejs_conductor/dist
	@$(RM) -rf app_spec/dist
	@for cargo in $$( find . -name 'Cargo.toml' ); do \
	    echo -e "\033[0;93m## 'cargo clean' in $${cargo%/*} ##\033[0m"; \
	    ( cd $${cargo%/*} && $(CARGO) clean ); \
	done

# clean up the extraneous "C" binding test files
${C_BINDING_CLEAN}:
	-@$(RM) $@
