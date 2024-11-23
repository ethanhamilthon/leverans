import os
import subprocess
from github import Github

# Настройки
REPO = "ethanhamilthon/leverans"  # Замените на ваш репозиторий
VERSION = "v0.1.5"  # Версия релиза
PACKAGE_NAME = "lev"  # Замените на имя вашего пакета
BUILD_TARGETS = [
    ("x86_64-unknown-linux-gnu", "linux-amd64"),
    ("aarch64-unknown-linux-gnu", "linux-arm64"),
    ("x86_64-apple-darwin", "macos-amd64"),
    ("aarch64-apple-darwin", "macos-arm64"),
    ("x86_64-pc-windows-msvc", "windows-amd64"),
    ("aarch64-pc-windows-msvc", "windows-arm64"),
]

def build(target, output_name):
    print(f"Сборка для {target}...")
    subprocess.run(
        ["sudo", "cargo", "build", "--release", "--target", target, "-p", PACKAGE_NAME],
        check=True
    )
    target_dir = f"target/{target}/release"
    os.rename(os.path.join(target_dir, PACKAGE_NAME), output_name)

def upload_to_github():
    token = os.getenv("GITHUB_TOKEN")
    if not token:
        raise EnvironmentError("GITHUB_TOKEN не настроен")

    gh = Github(token)
    repo = gh.get_repo(REPO)
    
    # Создание релиза
    release = repo.create_git_release(
        VERSION,  # Тег релиза
        VERSION,  # Название релиза
        "Автоматический релиз",  # Описание
        draft=False,  # Если релиз в черновике
        prerelease=False  # Если релиз предварительный
    )

    print("Загрузка файлов...")
    for _, output_name in BUILD_TARGETS:
        with open(output_name, "rb") as asset_file:
            release.upload_asset(asset_file, content_type="application/octet-stream", name=output_name)
        print(f"Загружен файл {output_name}")

def main():
    if not os.path.exists("target"):
        os.mkdir("target")

    # Сборка бинарников для выбранного пакета
    for target, output_name in BUILD_TARGETS:
        build(target, output_name)

    # Загрузка на GitHub Release
    upload_to_github()
    print("Все файлы успешно загружены!")

if __name__ == "__main__":
    main()
