# holochain-rust Makefile
# currently only supports 'debug' builds

# run `make` to build all the libraries and binaries
# run `make test` to execute all the tests
# run `make clean` to clean up the build environment

all: main

RUSTUP_DEFAULT_TOOLCHAIN ?= $(CORE_RUST_VERSION)
CORE_RUST_VERSION ?= nightly-2018-06-01
TOOLS_RUST_VERSION ?= nightly-2018-07-17
CARGO = cargo $(CARGO_ARGS) +$(CORE_RUST_VERSION)
CARGO_TOOLS = cargo $(CARGO_ARGS) +$(TOOLS_RUST_VERSION)

# list all the "C" binding tests that have been written
C_BINDING_DIRS = $(sort $(dir $(wildcard c_binding_tests/*/)))

# list all the "C" binding test executables that should be produced
C_BINDING_TESTS = $(foreach dir,$(C_BINDING_DIRS),target/debug/c_binding_tests/$(shell basename $(dir))/test_executable)

# list all the extraneous files that will be generated when running tests
C_BINDING_CLEAN = $(foreach dir,$(C_BINDING_DIRS),$(dir)Makefile $(dir).qmake.stash)

# build artifact / dependency checking is handled by our sub-tools
# we can just try to build everything every time, it should be efficient
.PHONY: main \
	c_binding_tests ${C_BINDING_DIRS} \
	test ${C_BINDING_TESTS} \
        test_non_c \
	clean ${C_BINDING_CLEAN}

# idempotent install rustup with the default toolchain set for Holochain core
# best for green fields Rust installation
.PHONY: install_rustup
install_rustup:
	if ! which rustup ; then \
		curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain $(RUSTUP_DEFAULT_TOOLCHAIN) -y; \
	fi
	export PATH=${HOME}/.cargo/bin:${PATH}
	rustc --version

# idempotent installation of core toolchain
.PHONY: core_toolchain
core_toolchain: install_rustup
	rustup toolchain install ${CORE_RUST_VERSION}

# idempotent installation of tools toolchain
.PHONY: tools_toolchain
tools_toolchain: install_rustup
	rustup toolchain install ${TOOLS_RUST_VERSION}

# idempotent addition of wasm target
.PHONY: ensure_wasm_target
ensure_wasm_target: core_toolchain
	rustup target add wasm32-unknown-unknown --toolchain ${CORE_RUST_VERSION}

# idempotent installation of development tooling
.PHONY: install_rust_tools
install_rust_tools: tools_toolchain
	# rust format
	if ! rustup component list --toolchain $(TOOLS_RUST_VERSION) | grep 'rustfmt-preview.*(installed)'; then \
		rustup component add --toolchain $(TOOLS_RUST_VERSION) rustfmt-preview; \
	fi
	# clippy
	if ! rustup component list --toolchain $(TOOLS_RUST_VERSION) | grep 'clippy-preview.*(installed)'; then \
		rustup component add --toolchain $(TOOLS_RUST_VERSION) clippy-preview; \
	fi

# idempotent installation of code coverage CI/testing tools
.PHONY: install_ci
install_ci: core_toolchain
	# tarpaulin (code coverage)
	if ! $(CARGO) install --list | grep 'cargo-tarpaulin'; then \
		RUSTFLAGS="--cfg procmacro2_semver_exempt" $(CARGO) install cargo-tarpaulin; \
	fi

# apply formatting / style guidelines, and build the rust project
main: fmt_check clippy build

# list all our found "C" binding tests
c_binding_tests: ${C_BINDING_DIRS}

# build all our found "C" binding tests
${C_BINDING_DIRS}:
	qmake -o $@Makefile $@qmake.pro
	cd $@; $(MAKE)

# execute all tests, both rust and "C" bindings
test: test_non_c c_binding_tests ${C_BINDING_TESTS}

test_non_c: main
	RUSTFLAGS="-D warnings" $(CARGO) test

test_c_ci: c_binding_tests ${C_BINDING_TESTS}

.PHONY: wasm_build
wasm_build: ensure_wasm_target
	cd core/src/nucleus/wasm-test && $(CARGO) +$(CORE_RUST_VERSION) build --target wasm32-unknown-unknown
	cd core_api/wasm-test/round_trip && $(CARGO) +$(CORE_RUST_VERSION) build --target wasm32-unknown-unknown
	cd core_api/wasm-test/commit && $(CARGO) +$(CORE_RUST_VERSION) build --target wasm32-unknown-unknown

.PHONY: build
build: wasm_build
	$(CARGO) build --all

cov:
	$(CARGO) tarpaulin --all --out Xml

fmt_check: install_rust_tools
	$(CARGO_TOOLS) fmt -- --check

clippy:
	$(CARGO_TOOLS) clippy -- -A needless_return

fmt:
	$(CARGO_TOOLS) fmt

# execute all the found "C" binding tests
${C_BINDING_TESTS}:
	$@

# clean up the target directory and all extraneous "C" binding test files
clean: ${C_BINDING_CLEAN}
	-@$(RM) -rf target

# clean up the extraneous "C" binding test files
${C_BINDING_CLEAN}:
	-@$(RM) $@
