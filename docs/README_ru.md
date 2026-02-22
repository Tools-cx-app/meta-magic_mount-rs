<div align="center">

# Метамодуль Magic Mount

[![Telegram][telegram-badge]][telegram-url]

</div>

[telegram-badge]: https://img.shields.io/badge/Group-blue?style=for-the-badge&logo=telegram&label=Telegram
[telegram-url]: https://t.me/mmrs_ci

Предоставляет возможности монтирования без использования системных файлов для KernelSU.

Автор будет обновлять этот проект реже из-за студенческих обязательств.

---

## Конфигурация

Путь к файлу конфигурации:

`/data/adb/magic_mount/config.toml`

Пример:

```toml
mountsource = "KSU"
umount = false
partitions = []
```

| Field | Описание |
| ------------- | -------------- |
| `mountsource` | Идентификатор источника монтирования, не затрагивающий системный раздел. По умолчанию `"KSU` для соответствия логике KernelSU. |
| `umount` | Попытка размонтирования (зависит от функции umount в KernelSU). |
| `partitions` | Список конкретных разделов, для выполнения операций без использования системного раздела, например, `"mi_ext"`, `"my_stock"`. |
| `tmpfsdir` | Временный путь, по умолчанию `/debug_ramdisk`. Эта опция опциоанальная.|

Конфигурация также может быть выполнена через WebUI. (рекомендуется).
---

## Разработка

Зависимости:

* Rust nightly toolchain
* Android NDK
* `cargo-ndk`
* Node.js / npm
* `pnpm` и `vite` как зависимости и фронтенд для webui


Переменные среды:
```shell
export ANDROID_NDK_HOME=<path/to/ndk>
export ANDROID_NDK_ROOT=$ANDROID_NDK_HOME
```

Билд:

```shell
cargo xtask b
```

Файлы билда будут находиться по следующему пути:
* `output/magic_mount_rs.zip`

## Благодарности

* [5ec1cff/KernelSU](https://github.com/5ec1cff/KernelSU/blob/52f1f575ce2bd0ca46ebf644fd00a838af9f344e/userspace/ksud/src/magic_mount.rs): оригинальная имплементация
* [YuzakiKokuban](https://github.com/YuzakiKokuban) модификации webui

## Лицензия

* [GPL-3.0 license](https://www.gnu.org/licenses/gpl-3.0.html)
