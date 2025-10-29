#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored messages
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_success() {
    echo -e "${BLUE}[SUCCESS]${NC} $1"
}

# Function to display help
show_help() {
    cat << EOF
Usage: $0 [OPTION]

Run GitHub Actions workflows locally using act to verify changes before committing.

Options:
    --all           Run all CI jobs (default)
    --format        Run format check only
    --check         Run clippy check only
    --test          Run unit tests only
    --examples      Run examples only
    --coverage      Run coverage check only
    --job <name>    Run specific job by name
    --list          List all available jobs
    --help          Display this help message

Examples:
    $0                  # Run all jobs
    $0 --all            # Run all jobs
    $0 --format         # Run format check only
    $0 --test           # Run unit tests only
    $0 --job coverage   # Run coverage job only
    $0 --list           # List available jobs

Requirements:
  - act must be installed (https://github.com/nektos/act)
  - Docker must be running
  - GitHub Actions workflow at .github/workflows/lorm.yml

Installation:
  # macOS
  brew install act

  # Linux
  curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

  # Or via Go
  go install github.com/nektos/act@latest

EOF
}

# Check if act is installed
check_act() {
    if ! command -v act &> /dev/null; then
        print_error "act is not installed."
        echo ""
        echo "Please install act to run GitHub Actions locally:"
        echo ""
        echo "macOS:"
        echo "  brew install act"
        echo ""
        echo "Linux:"
        echo "  curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash"
        echo ""
        echo "Via Go:"
        echo "  go install github.com/nektos/act@latest"
        echo ""
        echo "For more information: https://github.com/nektos/act"
        exit 1
    fi
}

# Check if Docker is running
check_docker() {
    if ! docker info &> /dev/null; then
        print_error "Docker is not running."
        echo ""
        echo "act requires Docker to run GitHub Actions locally."
        echo "Please start Docker and try again."
        exit 1
    fi
}

# List available jobs
list_jobs() {
    print_info "Available jobs in .github/workflows/lorm.yml:"
    echo ""
    echo "  - format      Format check (cargo fmt)"
    echo "  - check       Clippy linting (cargo clippy)"
    echo "  - test        Unit tests (cargo test)"
    echo "  - examples    Run all examples"
    echo "  - coverage    Code coverage with thresholds"
    echo ""
}

# Run specific job
run_job() {
    local job_name=$1
    print_info "Running job: $job_name"
    act -j "$job_name" --workflows .github/workflows/lorm.yml
}

# Run all jobs
run_all() {
    print_info "Running all CI jobs locally..."
    print_warning "This may take several minutes..."
    echo ""
    act --workflows .github/workflows/lorm.yml
}

# Main script
main() {
    local job=""
    local run_all_jobs=true

    # Parse arguments
    if [ $# -eq 0 ]; then
        run_all_jobs=true
    else
        case "$1" in
            --all)
                run_all_jobs=true
                ;;
            --format)
                job="format"
                run_all_jobs=false
                ;;
            --check)
                job="check"
                run_all_jobs=false
                ;;
            --test)
                job="test"
                run_all_jobs=false
                ;;
            --examples)
                job="examples"
                run_all_jobs=false
                ;;
            --coverage)
                job="coverage"
                run_all_jobs=false
                ;;
            --job)
                if [ $# -lt 2 ]; then
                    print_error "Missing job name after --job"
                    echo ""
                    show_help
                    exit 1
                fi
                job="$2"
                run_all_jobs=false
                ;;
            --list)
                list_jobs
                exit 0
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo ""
                show_help
                exit 1
                ;;
        esac
    fi

    # Check dependencies
    print_info "Checking dependencies..."
    check_act
    check_docker
    print_success "Dependencies OK"
    echo ""

    # Run jobs
    if [ "$run_all_jobs" = true ]; then
        run_all
    else
        run_job "$job"
    fi

    echo ""
    print_success "CI tests complete!"
    echo ""
    print_info "Your changes are ready to commit and push!"
}

main "$@"
