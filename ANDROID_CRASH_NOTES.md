# Vinteria Android: сбой при запуске (заметки для Claude и ChatGPT)

Дата: 2026-11-21. Проект: `C:\Users\ADMIN\Desktop\Vinteria` (Rust, eframe/egui, cargo-apk).

## Проблема
APK `com.winteria.daw` версии 1.0.6 на телефоне (Realme RMX5303, Android 15, arm64) вылетает сразу после запуска:

```
java.lang.UnsatisfiedLinkError: dlopen failed: cannot locate symbol "__cxa_pure_virtual"
referenced by .../lib/arm64/libvinteria_daw.so   (NativeActivity.onCreate)
```

## Диагноз (проверено через ADB и llvm-readelf)
- В `libvinteria_daw.so` из APK 1.0.6 есть неразрешённые C++ символы (`__cxa_pure_virtual`, `_Znwm`, `__cxa_begin_catch`, `_ZNSt9exceptionD2Ev`) и зависимость от `libOpenSLES.so`.
- В списке NEEDED нет `libc++_shared.so`, статически C++ тоже не слинкован. Android не находит символы и роняет приложение.
- Вероятная причина: в 1.0.6 добавили нативный аудио-бэкенд на C++ (oboe через cpal или свой OpenSL ES) без линковки C++ runtime.
- Исходники именно 1.0.6 в этом репозитории отсутствуют. Здесь версия `1.0.0-beta.1`: на Android нет аудио (cpal только для не-Android, на Android заглушка `AudioIo` в `src/audio_engine/device.rs`), C++ в сборке нет.

## Что сделано в репозитории
1. `Cargo.toml`: `package = "com.winteria.daw"` (раньше `com.synthesia.daw`), `target_sdk_version = 35` (в SDK нет platform 34, есть 35 и 36).
2. `src/ui/account_ui.rs`: добавлен `#[cfg(target_os = "android")] use crate::rfd;` (без него Android-сборка не компилировалась: `rfd` на Android это заглушка в `src/lib.rs`).
3. Release-сборка (`cargo apk build --release`) теперь компилируется, но падает на подписи: нужен `[package.metadata.android.signing.release]` с keystore. Без него собирать `cargo apk build` (debug-подпись, APK в `target/debug/apk/`).
4. На телефон новая сборка НЕ ставилась и запуск не проверялся: сборку остановили, её сделает пользователь/ChatGPT. Сломанная 1.0.6 остаётся на телефоне.

Репозиторий: https://github.com/ffight763-glitch/Vinteria-git3 (ветка main). Обновления для кнопки Update: `ffight763-glitch/vinteria-updates` (см. `update_url.txt`).

## Как собирать и ставить
Переменные окружения: `ANDROID_HOME=C:\Users\ADMIN\Android\Sdk`, `ANDROID_NDK_ROOT=...\ndk\27.2.12479018`.

```bash
cargo apk build --release        # APK: target/release/apk/VinteriaDaw.apk
```

ADB не в PATH, лежит тут: `C:\Users\ADMIN\Desktop\Aos\scrcpy\scrcpy-win64-v5.0.1\adb.exe`.

```bash
adb devices -l
adb install -r target/release/apk/VinteriaDaw.apk
adb logcat -c
adb shell monkey -p com.winteria.daw 1
adb logcat -d -b crash
```

Если установка ругается на подпись (`INSTALL_FAILED_UPDATE_INCOMPATIBLE`), старая сломанная 1.0.6 подписана другим ключом: её нужно сначала удалить (`adb uninstall com.winteria.daw`). Данные сломанной версии при этом пропадут, спросить пользователя.

## Что осталось сделать
- Если когда-нибудь вернётся Android-аудио: слинковать C++ runtime. Либо положить `libc++_shared.so` из NDK 27 (`toolchains/llvm/prebuilt/windows-x86_64/sysroot/usr/lib/aarch64-linux-android/`) в APK, либо `-lc++_static` через `build.rs`/`.cargo/config.toml`. После сборки проверять: `llvm-readelf -d libvinteria_daw.so | grep NEEDED` и отсутствие неразрешённых `__cxa_*`.
- Найти исходники 1.0.6 (если они нужны), в этом репозитории их нет.
- Закоммитить правки (пока ничего не закоммичено).
- `vinteria_crash.log` (панику при печати в закрытый stderr, os error 232) это отдельная десктопная мелочь, к Android не относится.

## Статус телефона
Сломанная `com.winteria.daw` 1.0.6 удалена с телефона через `adb uninstall` (2026-11-21). На телефоне остался старый `com.synthesia.daw` (другая сборка, не трогали).

## Промпт для ChatGPT (скопировать целиком)

```
Ты помогаешь с Rust-проектом Vinteria DAW (egui/eframe, Android через cargo-apk).
Репозиторий: https://github.com/ffight763-glitch/Vinteria-git3 (ветка main).
Сначала прочитай ANDROID_CRASH_NOTES.md в корне репозитория.

Контекст: APK com.winteria.daw 1.0.6 падал при запуске на Android 15 (arm64):
UnsatisfiedLinkError: cannot locate symbol "__cxa_pure_virtual" в libvinteria_daw.so.
Причина: в .so есть неразрешённые C++ символы и зависимость от libOpenSLES.so, но не слинкован C++ runtime
(libc++_shared.so нет в NEEDED, статически тоже нет). Исходников 1.0.6 в репозитории нет, там 1.0.0-beta.1
без звука на Android (заглушка AudioIo в src/audio_engine/device.rs).

Задача:
1. Дай пошаговую инструкцию (Windows, PowerShell), как собрать debug-APK: cargo apk build
   (ANDROID_HOME=C:\Users\ADMIN\Android\Sdk, NDK 27.2.12479018; release требует keystore в
   [package.metadata.android.signing.release]).
2. Если нужен release: как создать keystore (keytool) и прописать его в Cargo.toml, не коммитя ключ в git.
3. Как установить: adb install -r target/debug/apk/VinteriaDaw.apk, затем adb logcat -d -b crash
   и проверить, что приложение запускается.
4. Если добавляется Android-звук на C++ (oboe/cpal): как слинковать C++ runtime
   (libc++_shared.so из NDK в APK или -lc++_static) и как проверить через llvm-readelf -d, что
   в NEEDED есть libc++_shared.so и нет неразрешённых __cxa_* символов.
Отвечай по-русски, команды давай в отдельных блоках кода.
```
