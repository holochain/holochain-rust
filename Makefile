# holochain-rust Makefile
# currently only supports 'debug' builds

# run `make` to build all the libraries and binaries
# run `make test` to execute all the tests
# run `make clean` to clean up the build environment

all: main

CARGO = cargo $(CARGO_ARGS)

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
main:
	make fmt_check
	make clippy
	make build

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

.PHONY: install_rust_wasm
install_rust_wasm:
	rustup toolchain install ${WASM_NIGHTLY}
	rustup target add wasm32-unknown-unknown --toolchain ${WASM_NIGHTLY}

.PHONY: wasm_build
wasm_build:
	cd core/src/nucleus/wasm-test && $(CARGO) +$(WASM_NIGHTLY) build --target wasm32-unknown-unknown
	cd core_api/wasm-test/round_trip && $(CARGO) +$(WASM_NIGHTLY) build --target wasm32-unknown-unknown
	cd core_api/wasm-test/commit && $(CARGO) +$(WASM_NIGHTLY) build --target wasm32-unknown-unknown

.PHONY: build
build:
	$(CARGO) build --all
	make wasm_build

cov:
	$(CARGO) tarpaulin --all --out Xml

fmt_check:
	$(CARGO) +$(TOOLS_NIGHTLY) fmt -- --check

clippy:
	$(CARGO) +$(TOOLS_NIGHTLY) clippy -- -A needless_return

fmt:
	$(CARGO) +$(TOOLS_NIGHTLY) fmt

# execute all the found "C" binding tests
${C_BINDING_TESTS}:
	$@

# clean up the target directory and all extraneous "C" binding test files
clean: ${C_BINDING_CLEAN}
	-@$(RM) -rf target

# clean up the extraneous "C" binding test files
${C_BINDING_CLEAN}:
	-@$(RM) $@
