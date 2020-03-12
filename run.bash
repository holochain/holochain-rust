#!/bin/bash

echo "# TEST_TIME(test:start) $(date -u '+%s')"
hc-test-app-spec 2>&1
echo "# TEST_TIME(test:end) $(date -u '+%s')"
