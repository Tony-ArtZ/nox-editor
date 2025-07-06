#!/bin/bash

# Nox Editor Installation Script
# Repository: https://github.com/Tony-ArtZ/nox-editor

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# ASCII Art Banner
print_banner() {
    echo -e "${PURPLE}"
    cat << "EOF"
    ███╗   ██╗ ██████╗ ██╗  ██╗    ███████╗██████╗ ██╗████████╗ ██████╗ ██████╗ 
    ████╗  ██║██╔═══██╗╚██╗██╔╝    ██╔════╝██╔══██╗██║╚══██╔══╝██╔═══██╗██╔══██╗
    ██╔██╗ ██║██║   ██║ ╚███╔╝     █████╗  ██║  ██║██║   ██║   ██║   ██║██████╔╝
    ██║╚██╗██║██║   ██║ ██╔██╗     ██╔══╝  ██║  ██║██║   ██║   ██║   ██║██╔══██╗
    ██║ ╚████║╚██████╔╝██╔╝ ██╗    ███████╗██████╔╝██║   ██║   ╚██████╔╝██║  ██║
    ╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝    ╚══════╝╚═════╝ ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝
EOF
    echo -e "${NC}"
    echo -e "${CYAN}    A modern terminal-based text editor written in Rust${NC}"
    echo -e "${WHITE}    ════════════════════════════════════════════════════════════════════════${NC}"
    echo
}

# Print colored output
print_step() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check system requirements
check_requirements() {
    print_step "Checking system requirements..."
    
    # Check for Git
    if ! command_exists git; then
        print_error "Git is not installed. Please install Git and try again."
        print_step "Ubuntu/Debian: sudo apt update && sudo apt install git"
        print_step "CentOS/RHEL: sudo yum install git"
        print_step "macOS: brew install git"
        exit 1
    fi
    
    # Check for Rust
    if ! command_exists rustc; then
        print_warning "Rust is not installed. Installing Rust..."
        install_rust
    else
        print_success "Rust is already installed ($(rustc --version))"
    fi
    
    # Check for Cargo
    if ! command_exists cargo; then
        print_error "Cargo is not installed. Please install Rust with Cargo and try again."
        exit 1
    fi
    
    print_success "All requirements met!"
}

# Install Rust
install_rust() {
    print_step "Installing Rust via rustup..."
    
    if command_exists curl; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    elif command_exists wget; then
        wget -qO- https://sh.rustup.rs | sh -s -- -y
    else
        print_error "Neither curl nor wget is available. Please install one of them and try again."
        exit 1
    fi
    
    # Source the cargo environment
    source "$HOME/.cargo/env"
    print_success "Rust installed successfully!"
}

# Get installation directory
get_install_dir() {
    # Default installation directory
    DEFAULT_INSTALL_DIR="$HOME/.local/bin"
    
    echo -e "${YELLOW}Choose installation directory:${NC}"
    echo -e "  ${WHITE}1)${NC} $DEFAULT_INSTALL_DIR (recommended)"
    echo -e "  ${WHITE}2)${NC} /usr/local/bin (system-wide, requires sudo)"
    echo -e "  ${WHITE}3)${NC} Custom directory"
    echo
    
    while true; do
        read -p "Enter your choice (1-3): " choice
        case $choice in
            1)
                INSTALL_DIR="$DEFAULT_INSTALL_DIR"
                break
                ;;
            2)
                INSTALL_DIR="/usr/local/bin"
                NEED_SUDO=true
                break
                ;;
            3)
                read -p "Enter custom directory path: " INSTALL_DIR
                if [[ -n "$INSTALL_DIR" ]]; then
                    break
                else
                    print_warning "Please enter a valid directory path."
                fi
                ;;
            *)
                print_warning "Please enter 1, 2, or 3."
                ;;
        esac
    done
    
    # Expand tilde to home directory
    INSTALL_DIR="${INSTALL_DIR/#\~/$HOME}"
    
    print_step "Installation directory: $INSTALL_DIR"
}

# Create temporary directory
create_temp_dir() {
    TEMP_DIR=$(mktemp -d)
    print_step "Created temporary directory: $TEMP_DIR"
    
    # Cleanup function
    cleanup() {
        print_step "Cleaning up temporary files..."
        rm -rf "$TEMP_DIR"
    }
    trap cleanup EXIT
}

# Clone repository
clone_repository() {
    print_step "Cloning Nox Editor repository..."
    
    cd "$TEMP_DIR"
    if git clone https://github.com/Tony-ArtZ/nox-editor.git; then
        print_success "Repository cloned successfully!"
    else
        print_error "Failed to clone repository. Please check your internet connection."
        exit 1
    fi
    
    cd nox-editor
}

# Build the project
build_project() {
    print_step "Building Nox Editor (this may take a few minutes)..."
    
    # Add progress indicator
    {
        cargo build --release 2>&1 | while IFS= read -r line; do
            echo "$line"
            if [[ "$line" == *"Compiling"* ]]; then
                echo -ne "${CYAN}⚡${NC} Building... "
            fi
        done
    } || {
        print_error "Build failed. Please check the error messages above."
        exit 1
    }
    
    print_success "Build completed successfully!"
}

# Install the binary
install_binary() {
    print_step "Installing Nox Editor to $INSTALL_DIR..."
    
    # Create installation directory if it doesn't exist
    if [[ "$NEED_SUDO" == "true" ]]; then
        sudo mkdir -p "$INSTALL_DIR"
        sudo cp target/release/nox-editor "$INSTALL_DIR/"
        sudo chmod +x "$INSTALL_DIR/nox-editor"
    else
        mkdir -p "$INSTALL_DIR"
        cp target/release/nox-editor "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/nox-editor"
    fi
    
    print_success "Nox Editor installed successfully!"
}

# Check if directory is in PATH
check_path() {
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        print_success "Installation directory is already in your PATH."
    else
        print_warning "Installation directory is not in your PATH."
        print_step "To use 'nox-editor' from anywhere, add this line to your shell profile:"
        print_step "    export PATH=\"$INSTALL_DIR:\$PATH\""
        echo
        print_step "For bash: echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
        print_step "For zsh:  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.zshrc"
        print_step "Then restart your terminal or run: source ~/.bashrc (or ~/.zshrc)"
    fi
}

# Main installation function
main() {
    clear
    print_banner
    
    echo -e "${WHITE}This script will install Nox Editor on your system.${NC}"
    echo -e "${WHITE}Press Enter to continue or Ctrl+C to cancel...${NC}"
    read -r
    
    check_requirements
    get_install_dir
    create_temp_dir
    clone_repository
    build_project
    install_binary
    check_path
    
    echo
    print_success "🎉 Nox Editor has been installed successfully!"
    echo
    print_step "You can now run it with:"
    echo -e "    ${GREEN}nox-editor${NC}           # Create a new file"
    echo -e "    ${GREEN}nox-editor <file>${NC}    # Open an existing file"
    echo
    print_step "For help and documentation, visit:"
    echo -e "    ${BLUE}https://github.com/Tony-ArtZ/nox-editor${NC}"
    echo
    print_step "Enjoy coding with Nox Editor! 🚀"
}

# Run the installation
main "$@"
