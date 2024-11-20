import os
import subprocess
import sys

APP_NAME = "lev"
BINARY_NAME = "lev"

try:
    subprocess.run(["cargo", "--version"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
except (subprocess.CalledProcessError, FileNotFoundError):
    print("Rust is not installed. Please install Rust using the following command: 'curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh'")
    sys.exit(1)

print(f"Compiling {APP_NAME}...")
try:
    subprocess.run(["sudo", "cargo", "build", "-p", APP_NAME, "--release"], check=True)
except subprocess.CalledProcessError:
    print(f"Error: project did not compile: {APP_NAME}")
    sys.exit(1)

BINARY_PATH = f"target/release/{BINARY_NAME}"
if not os.path.isfile(BINARY_PATH):
    print(f"Error: binary {BINARY_NAME} not found.")
    sys.exit(1)

print(f"Moving {BINARY_NAME} to /usr/local/bin...")
try:
    subprocess.run(["sudo", "mv", BINARY_PATH, "/usr/local/bin/"], check=True)
except subprocess.CalledProcessError:
    print(f"Error: failed to move {BINARY_NAME} to /usr/local/bin.")
    sys.exit(1)

print(f"Lev CLI has been installed to /usr/local/bin as '{BINARY_NAME}'")
