# Tsunami Dam Simulator (Bevy 0.15 + Avian3D 0.2)

3D-игра «Симулятор цунами: построй дамбу» на Rust с процедурной графикой.

---

## Способ 1: Сборка APK полностью с телефона (через GitHub Actions)

Собирать Bevy локально на телефоне тяжело (нужен Android NDK, Gradle и 8+ ГБ ОЗУ). Проще и быстрее собрать APK бесплатно в облаке GitHub:

1. Распакуй `tsunami_dam.zip` на телефоне (в любом файловом менеджере или в Termux: `unzip tsunami_dam.zip -d tsunami_dam`).
2. Создай новый пустой репозиторий на GitHub (например, `tsunami-dam`).
3. Залей файлы проекта в репозиторий:
   - **Через Termux (быстрее всего):**
     ```bash
     cd tsunami_dam
     git init
     git branch -M main
     git add .
     git commit -m "init"
     git remote add origin https://github.com/<ТВОЙ_НИК>/tsunami-dam.git
     git push -u origin main
     ```
   - Либо загрузи файлы вручную через браузер (главное — сохранить папку `.github/workflows/build-apk.yml`).
4. Открой вкладку **Actions** в своём репозитории на GitHub → выбери **Build Android APK**.
5. Дождись окончания сборки (первый раз ~6–10 минут, дальше с кешем быстрее).
6. Внизу страницы завершённого запуска нажми на артефакт **`tsunami-dam-apk`** — скачается архив с готовым `app-release.apk`.
7. Распакуй и установи `app-release.apk` на телефон.

---

## Способ 2: Запуск и сборка на ПК (Windows)

Быстрый тест игры на ПК:
```powershell
cargo check
cargo run --bin tsunami_dam_pc
```

Сборка APK на ПК (нужны Android SDK, NDK, Java 17 и `cargo install cargo-ndk`):
```powershell
.\build_apk.ps1
```

---

## Управление в игре

- **Тап по сетке (или ЛКМ)** — поставить выбранный блок (при повторном тапе блоки ставятся друг на друга вверх до 5 ярусов).
- **Режим DELETE + тап** — удалить верхний блок в клетке и вернуть бюджет.
- **Свайп 1 пальцем (или ПКМ/перетаскивание ЛКМ)** — вращение камеры вокруг дамбы.
- **Сведение 2 пальцев (или колесо мыши)** — зум.
- **START** — запуск волны цунами.
- **RESET** — полная очистка дамбы.
