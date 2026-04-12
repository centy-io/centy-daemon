#!/bin/bash
#
# E2E Test Runner for Docker
#
# This script builds and runs the e2e tests in a Docker container.
# It compiles the daemon and runs all tests with snapshot support.
#
# All e2e testing files are encapsulated in the e2e folder.
#
# Usage:
#   ./scripts/run-docker.sh              # Run all tests
#   ./scripts/run-docker.sh --watch      # Run in watch mode (dev container)
#   ./scripts/run-docker.sh --build-only # Only build, don't run tests
#   ./scripts/run-docker.sh --clean      # Clean up Docker resources
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
E2E_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Parse arguments
WATCH_MODE=false
BUILD_ONLY=false
CLEAN_MODE=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --watch|-w)
            WATCH_MODE=true
            shift
            ;;
        --build-only|-b)
            BUILD_ONLY=true
            shift
            ;;
        --clean|-c)
            CLEAN_MODE=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "E2E Docker Test Runner"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --watch, -w       Run in watch mode (interactive dev container)"
            echo "  --build-only, -b  Only build the Docker image, don't run tests"
            echo "  --clean, -c       Clean up Docker resources"
            echo "  --verbose, -v     Verbose output"
            echo "  --help, -h        Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

cd "$E2E_DIR"

# Clean mode
if [ "$CLEAN_MODE" = true ]; then
    log_info "Cleaning up Docker resources..."
    docker compose down --volumes --remove-orphans 2>/dev/null || true
    docker rmi centy-e2e-tests 2>/dev/null || true
    rm -rf test-output
    log_success "Cleanup complete"
    exit 0
fi

# Create output directory
mkdir -p test-output/snapshots

# Build the Docker image
log_info "Building E2E Docker image..."
if [ "$VERBOSE" = true ]; then
    docker compose build e2e-tests
else
    docker compose build e2e-tests --quiet
fi
log_success "Docker image built successfully"

if [ "$BUILD_ONLY" = true ]; then
    log_success "Build-only mode: skipping test execution"
    exit 0
fi

# Watch mode - run dev container
if [ "$WATCH_MODE" = true ]; then
    log_info "Starting development container..."
    log_info "You can run tests manually with: pnpm test:watch"
    docker compose --profile dev run --rm e2e-dev
    exit 0
fi

# Run the tests
log_info "Running E2E tests in Docker..."
echo ""

# Run tests with abort-on-container-exit
TEST_EXIT_CODE=0
docker compose up --abort-on-container-exit e2e-tests || TEST_EXIT_CODE=$?

# Clean up containers
docker compose down --volumes 2>/dev/null || true

# Check results
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo ""
    log_success "All E2E tests passed!"

    # Show snapshot summary if available
    if [ -d "test-output/snapshots" ] && [ "$(ls -A test-output/snapshots 2>/dev/null)" ]; then
        log_info "Snapshots saved to: test-output/snapshots/"
        ls -la test-output/snapshots/
    fi
else
    echo ""
    log_error "E2E tests failed with exit code: $TEST_EXIT_CODE"
    exit $TEST_EXIT_CODE
fi
