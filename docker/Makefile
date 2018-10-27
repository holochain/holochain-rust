# holochain-rust Makefile
# currently only supports 'debug' builds

# run `make` to build all the libraries and binaries
# run `make test` to execute all the tests
# run `make clean` to clean up the build environment

all: main

CORE_RUST_VERSION ?= nightly-2018-10-12
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

# apply formatting / style guidelines, and build the rust project
main: fmt_check clippy build

# idempotent install rustup with the default toolchain set for tooling
# best for CI based on tools only
.PHONY: install_rustup
install_rustup:
	if ! which rustup ; then \
		curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain $(CORE_RUST_VERSION) -y; \
	fi
	export PATH="${HOME}/.cargo/bin:${PATH}"

# idempotent install rustup with the default toolchain set for Holochain core
# best for green fields Rust installation
.PHONY: install_rustup_tools
install_rustup_tools:
	if ! which rustup ; then \
		curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain $(TOOLS_RUST_VERSION) -y; \
	fi
	export PATH="${HOME}/.cargo/bin:${PATH}"

# idempotent installation of libzmq system library
# note, this is complicated by our use of travis-ci ubuntu trusty
# we need to install a newer version than is otherwise available
.PHONY: install_system_libzmq
install_system_libzmq:
	if ! (pkg-config libzmq --libs) ; then \
		if ! which apt-get ; then \
			if which brew ; then \
				echo "\033[0;93m## Attempting to install zmq using homebrew ##\033[0m"; \
				brew install zmq \
			else \
				echo "\033[0;93m## libzmq couldn't be installed, build probably won't work\033[0m"; \
			fi; \
		else \
			if [ "x${TRAVIS}" = "x" ]; then \
				echo "\033[0;93m## Attempting to install libzmq3-dev with apt-get ##\033[0m"; \
				sudo apt-get install -y libzmq3-dev; \
			else \
				echo "\033[0;93m## Attempting to install libzmq3-dev on UBUNTU TRUSTY ##\033[0m"; \
				echo "deb http://download.opensuse.org/repositories/network:/messaging:/zeromq:/release-stable/xUbuntu_14.04/ ./" >> /etc/apt/sources.list; \
				wget https://download.opensuse.org/repositories/network:/messaging:/zeromq:/release-stable/xUbuntu_14.04/Release.key -O- | sudo apt-key add; \
				sudo apt-get update -qq; \
				sudo apt-get install libzmq3-dev; \
			fi; \
		fi; \
	fi; \

# idempotent install of any required system libraries
.PHONY: install_system_libs
install_system_libs: install_system_libzmq

# idempotent installation of core toolchain
.PHONY: core_toolchain
core_toolchain: install_rustup install_system_libs
	rustup toolchain install ${CORE_RUST_VERSION}

# idempotent installation of tools toolchain
.PHONY: tools_toolchain
tools_toolchain: install_rustup_tools install_system_libs
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

.PHONY: install_mdbook
install_mdbook: tools_toolchain
	if ! $(CARGO_TOOLS) install --list | grep 'mdbook'; then \
		$(CARGO_TOOLS) install mdbook --vers "^0.2.2"; \
	fi

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

c_build: core_toolchain
	cd dna_c_binding && $(CARGO) build

test_c_ci: c_build c_binding_tests ${C_BINDING_TESTS}

.PHONY: wasm_build
wasm_build: ensure_wasm_target
	cd core/src/nucleus/wasm-test && $(CARGO) build --release --target wasm32-unknown-unknown
	cd core/src/nucleus/actions/wasm-test && $(CARGO) build --release --target wasm32-unknown-unknown
	cd core_api/wasm-test/round_trip && $(CARGO) build --release --target wasm32-unknown-unknown
	cd core_api/wasm-test/commit && $(CARGO) build --release --target wasm32-unknown-unknown
	cd hdk-rust/wasm-test && $(CARGO) build --release --target wasm32-unknown-unknown
	cd wasm_utils/wasm-test/integration-test && $(CARGO) build --release --target wasm32-unknown-unknown

.PHONY: build
build: core_toolchain wasm_build
	$(CARGO) build --all

code_coverage: core_toolchain wasm_build install_ci
	$(CARGO) tarpaulin --timeout 600 --all --out Xml --skip-clean -v -e holochain_core_api_c_binding -e hdk

fmt_check: install_rust_tools
	$(CARGO_TOOLS) fmt -- --check

clippy: install_rust_tools
	$(CARGO_TOOLS) clippy -- -A needless_return --A useless_attribute

fmt: install_rust_tools
	$(CARGO_TOOLS) fmt

# execute all the found "C" binding tests
${C_BINDING_TESTS}:
	$@

# clean up the target directory and all extraneous "C" binding test files
clean: ${C_BINDING_CLEAN}
	-@$(RM) -rf target
	-@$(RM) -rf wasm_utils/wasm-test/integration-test/target

# clean up the extraneous "C" binding test files
${C_BINDING_CLEAN}:
	-@$(RM) $@
