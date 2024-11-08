import sys
import subprocess

def check_command(command, error_message):
    try:
        subprocess.run(command, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except subprocess.CalledProcessError:
        print(error_message)
        sys.exit(1)

def main():
    if len(sys.argv) < 2:
        print(sys.argv)
        print("Usage: python script.py <version>")
        sys.exit(1)

    version = sys.argv[1]

    # Check if docker is installed
    check_command(['docker', '--version'], "Docker is not installed. Please install Docker and try again.")

    # Check if docker buildx is installed
    check_command(['docker', 'buildx'], "Docker Buildx is not installed. Please install Docker Buildx and try again.")

    # Check if login to docker hub is successful
    try:
        subprocess.run(['docker', 'login'], check=True)
    except subprocess.CalledProcessError:
        print("Docker login failed. Please check your credentials and try again.")
        sys.exit(1)

    print(f"Building Docker image: leverans/manager:{version}")
    subprocess.run(['docker', 'buildx', 'build', '--platform', 'linux/amd64,linux/arm64', '-t', f'leverans/manager:{version}', '--push', '.'], check=True)

if __name__ == "__main__":
    main()

