# holochain-rust Makefile
# currently only supports 'debug' builds

.PHONY: all build install help

all: build

build: build_holochain build_cli build_nodejs

install: build install_cli

help:
	@echo "run 'make' to build all the libraries and binaries, and the nodejs bin-package"
	@echo "run 'make install' to build and install all the libraries and binaries, and the nodejs bin-package"
	@echo "run 'make test' to execute all the tests"
	@echo "run 'make test_app_spec' to build and test app_spec API tests"
	@echo "run 'make clean' to clean up the build environment"
	@echo "run 'make test_holochain' to test holochain builds"
	@echo "run 'make test_cli' to build and test the command line tool builds"
	@echo "run 'make install_cli' to build and install the command line tool builds"
	@echo "run 'make build_conductor_wasm' to build the wasm light-client browser conductor"
	@echo "run 'make test-something' to run cargo tests matching 'something'"

SHELL = /bin/bash
CORE_RUST_VERSION ?= nightly-2019-07-14
TOOLS_RUST_VERSION ?= nightly-2019-07-14
CARGO = RUSTFLAGS="-Z external-macro-backtrace -D warnings" RUST_BACKTRACE=1 rustup run $(CORE_RUST_VERSION) cargo $(CARGO_ARGS)
CARGO_TOOLS = RUSTFLAGS="-Z external-macro-backtrace -D warnings" RUST_BACKTRACE=1 rustup run $(TOOLS_RUST_VERSION) cargo $(CARGO_ARGS)
CARGO_TARPULIN_INSTALL = RUSTFLAGS="--cfg procmacro2_semver_exempt -D warnings" RUST_BACKTRACE=1 cargo $(CARGO_ARGS) +$(CORE_RUST_VERSION)
OPENSSL_STATIC = 1


# build artifact / dependency checking is handled by our sub-tools
# we can just try to build everything every time, it should be efficient
.PHONY: lint \
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


# execute all tests: holochain, command-line tools, app spec, nodejs conductor, and "C" bindings
test: test_holochain test_cli test_app_spec c_binding_tests 

test_holochain: build_holochain
	cd crates/cli && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc
	cd crates/conductor_api && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc
	cd crates/conductor_lib && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc
	cd crates/core && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc 
	cd crates/core_types && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc
	cd crates/dpki && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc
	cd crates/hdk && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc 
	cd crates/hdk_v2 && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc
	cd crates/holochain && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc 
	cd crates/net && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc 
	cd crates/wasm_utils && RUSTFLAGS="-D warnings" $(CARGO) test --all --exclude hc

# Execute cargo tests matching %
# Eg. make test-stacked will run "cargo test stacked"
test-%: build-$*
	cd crates/$* && RUSTFLAGS="-D warnings" $(CARGO) test $* -- --nocapture

#Execute test based on crate
build-%: 
	cd crates/$* && RUSTFLAGS="-D warnings" $(CARGO) build 

test_cli: build_cli
	@echo -e "\033[0;93m## Testing hc command... ##\033[0m"
	cd crates/cli && RUSTFLAGS="-D warnings" $(CARGO) test

test_app_spec: RUST_VERSION=$(CORE_RUST_VERSION)
test_app_spec: version_rustup ensure_wasm_target install_cli build_rust_conductor
	@echo -e "\033[0;93m## Testing app_spec... ##\033[0m"
	( cd app_spec && ./build_and_test.sh )

build_nodejs_conductor: RUST_VERSION=$(CORE_RUST_VERSION)
build_nodejs_conductor: version_rustup core_toolchain
	@echo -e "\033[0;93m## Building nodejs_conductor... ##\033[0m"
	./scripts/build_nodejs_conductor.sh

build_rust_conductor: RUST_VERSION=$(CORE_RUST_VERSION)
build_rust_conductor: version_rustup core_toolchain
	@echo -e "\033[0;93m## Building rust conductor... ##\033[0m"
	$(CARGO) build -p holochain --release && $(CARGO) install -f --path conductor


.PHONY: wasm_build
wasm_build: ensure_wasm_target
	@echo -e "\033[0;93m## Building wasm targets... ##\033[0m"
	cd crates/core/src/nucleus/actions/wasm-test && $(CARGO) build --release --target wasm32-unknown-unknown
	cd crates/conductor_lib/wasm-test && $(CARGO) build --release --target wasm32-unknown-unknown
	cd crates/conductor_lib/test-bridge-caller && $(CARGO) build --release --target wasm32-unknown-unknown
	cd crates/hdk/wasm-test && $(CARGO) build --release --target wasm32-unknown-unknown
	cd crates/wasm_utils/wasm-test/integration-test && $(CARGO) build --release --target wasm32-unknown-unknown

.PHONY: install_wasm_bindgen_cli
install_wasm_bindgen_cli:
	@if ! $(CARGO_TOOLS) install --list | grep -q 'wasm-bindgen-cli v0.2.32'; then \
		echo -e "\033[0;93m## Installing wasm_bindgen_cli ##\033[0m"; \
		$(CARGO_TOOLS) install --force wasm-bindgen-cli --vers "0.2.32"; \
	fi

.PHONY: build_holochain
build_holochain: wasm_build
	@echo -e "\033[0;93m## Building holochain... ##\033[0m"
	cd crates/cli && $(CARGO) build 
	cd crates/conductor_api && $(CARGO) build 
	cd crates/conductor_lib && $(CARGO) build 
	cd crates/core && $(CARGO) 
	cd crates/core_types && $(CARGO) build 
	cd crates/dpki && $(CARGO) build 
	cd crates/hdk && $(CARGO) build 
	cd crates/hdk_v2 && $(CARGO) build 
	cd crates/holochain && $(CARGO) build 
	cd crates/net && $(CARGO) build 
	cd crates/wasm_utils && $(CARGO) build 

.PHONY: build_cli
build_cli: core_toolchain ensure_wasm_target
	@echo -e "\033[0;93m## Building hc command... ##\033[0m"
	cd crates/cli && $(CARGO) build -p hc

.PHONY: build_nodejs
build_nodejs:
	@echo -e "\033[0;93m## Building nodejs interface... ##\033[0m"
	cd nodejs_conductor && npm run compile && mkdir -p bin-package && cp native/index.node bin-package

.PHONY: install_cli
install_cli: build_cli
	@echo -e "\033[0;93m## Installing hc command... ##\033[0m"
	cd crates/cli && $(CARGO) install -f --path .

.PHONY: build_conductor_wasm
build_conductor_wasm: ensure_wasm_target install_wasm_bindgen_cli
	$(CARGO) build --release -p holochain_conductor_wasm --target wasm32-unknown-unknown
	wasm-bindgen target/wasm32-unknown-unknown/release/holochain_conductor_wasm.wasm --out-dir conductor_wasm/npm_package/gen --nodejs

.PHONY: code_coverage
code_coverage: core_toolchain wasm_build install_ci
	$(CARGO) tarpaulin --ignore-tests --timeout 600 --all --out Xml --skip-clean -v -e holochain_core_api_c_binding -e hdk -e hc -e holochain_json_derive

.PHONY: code_coverage_crate
code_coverage_crate: core_toolchain wasm_build install_ci
	$(CARGO) tarpaulin --ignore-tests --timeout 600 --skip-clean -v -p $(CRATE)

fmt_check: install_rust_tools
	$(CARGO_TOOLS) fmt -- --check

clippy: install_rust_tools
	$(CARGO_TOOLS) clippy -- -A clippy::needless_return --A clippy::useless_attribute

fmt: install_rust_tools
	$(CARGO_TOOLS) fmt


# clean up the target directory and all extraneous "C" binding test files
clean: 
	@for target in $$( find . -type d -a -name 'target' ); do \
	    echo -e "\033[0;93m## Removing $${target} ##\033[0m"; \
	    $(RM) -rf $${target}; \
        done
	@$(RM) -rf nodejs_conductor/dist
	@$(RM) -rf app_spec/dist
	@for cargo in $$( find . -name 'Cargo.toml' ); do \
	    echo -e "\033[0;93m## 'cargo clean' in $${cargo%/*} ##\033[0m"; \
	    ( cd $${cargo%/*} && cargo clean ); \
	done

