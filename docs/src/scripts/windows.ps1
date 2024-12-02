# Parameters
$url = "https://github.com/ethanhamilthon/leverans/releases/download/vacac/86_64-pc-windows-msvc.exe"
$destination = "$env:USERPROFILE\Downloads\leverans.exe" # Path to save the downloaded file
$installPath = "C:\Program Files\leverans" # Installation directory
$executablePath = "$installPath\leverans.exe"
$symlinkPath = "$installPath\lev.exe" # Symlink path

# Create the installation directory if it doesn't exist
if (-not (Test-Path $installPath)) {
    New-Item -ItemType Directory -Path $installPath -Force
}

# Download the binary
Write-Host "Downloading file from $url..."
Invoke-WebRequest -Uri $url -OutFile $destination -UseBasicParsing

# Move the file to the installation directory
Write-Host "Moving binary to $installPath..."
Move-Item -Path $destination -Destination $executablePath -Force

# Create a symlink with the desired name
Write-Host "Creating symlink as 'lev'..."
if (-not (Test-Path $symlinkPath)) {
    New-Item -ItemType SymbolicLink -Path $symlinkPath -Target $executablePath
} else {
    Write-Host "'lev' symlink already exists."
}

# Add the directory to PATH if it is not already present
$currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::Machine)
if (-not $currentPath.Contains($installPath)) {
    Write-Host "Adding $installPath to PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$installPath", [EnvironmentVariableTarget]::Machine)
}

Write-Host "Installation completed. You can now run 'lev' from the command line."
