import os
import subprocess
import sys

# Имя проекта (пакета) в workspace
APP_NAME = "lev"
# Новое имя бинарника
BINARY_NAME = "lev"

# Проверка, указано ли имя проекта
if not APP_NAME:
    print("Ошибка: укажите имя проекта. Например: python install.py myproject")
    sys.exit(1)

# Проверяем, установлен ли Rust
try:
    subprocess.run(["cargo", "--version"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
except (subprocess.CalledProcessError, FileNotFoundError):
    print("Rust не установлен. Установите его с помощью 'curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh'")
    sys.exit(1)

# Компилируем указанный проект из workspace
print(f"Компиляция проекта {APP_NAME}...")
try:
    subprocess.run(["sudo", "cargo", "build", "-p", APP_NAME, "--release"], check=True)
except subprocess.CalledProcessError:
    print(f"Ошибка: не удалось скомпилировать проект {APP_NAME}")
    sys.exit(1)

# Проверяем, что бинарник успешно создан
BINARY_PATH = f"target/release/{BINARY_NAME}"
if not os.path.isfile(BINARY_PATH):
    print(f"Ошибка: бинарник {BINARY_NAME} не создан.")
    sys.exit(1)

# Перемещаем бинарник в /usr/local/bin
print(f"Перемещение бинарника {BINARY_NAME} в /usr/local/bin...")
try:
    subprocess.run(["sudo", "mv", BINARY_PATH, "/usr/local/bin/"], check=True)
except subprocess.CalledProcessError:
    print(f"Ошибка: не удалось переместить бинарник в /usr/local/bin.")
    sys.exit(1)

print(f"Установка завершена. Теперь вы можете использовать '{BINARY_NAME}' в CLI.")
