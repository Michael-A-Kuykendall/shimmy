#!/usr/bin/env python3
"""
Shimmy Python Wrapper

This package provides a Python interface to the Shimmy binary,
the 5MB alternative to Ollama for local AI inference.
"""

import os
import sys
import subprocess
import platform
import urllib.request
import urllib.error
import tarfile
import zipfile
import stat
from pathlib import Path

__version__ = "0.1.0"

GITHUB_REPO = "Michael-A-Kuykendall/shimmy"
BINARY_NAME = "shimmy.exe" if platform.system() == "Windows" else "shimmy"

def get_platform_info():
    """Get platform and architecture information for binary download."""
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    # Map platform names
    platform_map = {
        'windows': 'windows',
        'darwin': 'darwin',
        'linux': 'linux'
    }
    
    # Map architecture names
    arch_map = {
        'x86_64': 'amd64',
        'amd64': 'amd64',
        'arm64': 'arm64',
        'aarch64': 'arm64'
    }
    
    if system not in platform_map:
        raise RuntimeError(f"Unsupported platform: {system}")
    
    if machine not in arch_map:
        raise RuntimeError(f"Unsupported architecture: {machine}")
    
    return platform_map[system], arch_map[machine]

def get_binary_path():
    """Get the path to the shimmy binary."""
    package_dir = Path(__file__).parent
    bin_dir = package_dir / "bin"
    return bin_dir / BINARY_NAME

def download_binary():
    """Download the appropriate shimmy binary for this platform."""
    platform_name, arch = get_platform_info()
    
    # Construct download URL
    version = __version__
    if platform_name == "windows":
        filename = f"shimmy-{version}-{platform_name}-{arch}.zip"
        download_url = f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/{filename}"
    else:
        filename = f"shimmy-{version}-{platform_name}-{arch}.tar.gz"
        download_url = f"https://github.com/{GITHUB_REPO}/releases/download/v{version}/{filename}"
    
    print(f"Downloading Shimmy binary from {download_url}")
    
    # Create bin directory
    bin_dir = Path(__file__).parent / "bin"
    bin_dir.mkdir(exist_ok=True)
    
    # Download archive
    archive_path = bin_dir / filename
    try:
        urllib.request.urlretrieve(download_url, archive_path)
    except urllib.error.URLError as e:
        raise RuntimeError(f"Failed to download binary: {e}")
    
    # Extract binary
    binary_path = bin_dir / BINARY_NAME
    
    try:
        if filename.endswith('.zip'):
            with zipfile.ZipFile(archive_path, 'r') as zip_file:
                zip_file.extract(BINARY_NAME, bin_dir)
        else:
            with tarfile.open(archive_path, 'r:gz') as tar_file:
                tar_file.extract(BINARY_NAME, bin_dir)
        
        # Make executable on Unix systems
        if platform.system() != "Windows":
            binary_path.chmod(binary_path.stat().st_mode | stat.S_IEXEC)
        
        print(f"Shimmy binary installed at {binary_path}")
        
    finally:
        # Clean up archive
        if archive_path.exists():
            archive_path.unlink()

def ensure_binary():
    """Ensure the shimmy binary is available, downloading if necessary."""
    binary_path = get_binary_path()
    
    if not binary_path.exists():
        print("Shimmy binary not found, downloading...")
        download_binary()
    
    return binary_path

def run_shimmy(args=None):
    """Run the shimmy binary with the given arguments."""
    binary_path = ensure_binary()
    
    if args is None:
        args = sys.argv[1:]
    
    try:
        result = subprocess.run([str(binary_path)] + args, check=True)
        return result.returncode
    except subprocess.CalledProcessError as e:
        return e.returncode
    except FileNotFoundError:
        print("Error: Shimmy binary not found. Try reinstalling the package.")
        return 1

def main():
    """Main entry point for the shimmy command."""
    return run_shimmy()

if __name__ == "__main__":
    sys.exit(main())
