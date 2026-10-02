<div align="center">

![An implementation of a metamodule](./mmrs.svg)

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
extra_mount = []
```

| Field | Описание |
| ------------- | -------------- |
| `mountsource` | Идентификатор источника монтирования, не затрагивающий системный раздел. По умолчанию `"KSU` для соответствия логике KernelSU. |
| `umount` | Попытка размонтирования (зависит от функции umount в KernelSU). |
| `partitions` | Список конкретных разделов, для выполнения операций без использования системного раздела, например, `"mi_ext"`, `"my_stock"`. |
| `tmpfsdir` | Временный путь, по умолчанию `/debug_ramdisk`. Эта опция опциоанальная.|

`extra_mount` по умолчанию равен `[]`. Например, `extra_mount = ["my_product"]` объединяет содержимое `<модуль>/my_product/` с `/my_product/`, а не читает `<модуль>/system/my_product/`. Целевой каталог должен существовать, а `/system/my_product` должен полностью отсутствовать, включая висячие символические ссылки. Указывайте только имена разделов без пути; недопустимые записи пропускаются с сообщением в журнале. Поддерживаются модули без `system/`; маркеры `disable`, `remove` и `skip_mount` сохраняют действие. После изменения конфигурации перезагрузите устройство.

Настройки также можно изменить на странице конфигурации WebUI. MiuiX и MD3 предоставляют редактор «Разделы верхнего уровня модуля» для добавления и удаления имён разделов `extra_mount`. Сохраните настройки и перезагрузите устройство.
---

## Пользовательские правила

Файл правил: `/data/adb/magic_mount/custom`

```text
# Игнорировать исходный файл из модуля
ignore /data/adb/modules/example/system/app/Example.apk

# Примонтировать источник к цели только для чтения
bind "/data/local/tmp/source file" "/system/etc/target file"

# Рекурсивно подключить другие файлы; add и file эквивалентны
file /data/adb/magic_mount/extra.rules
add /data/adb/magic_mount/more.rules
```

Поддерживаются одинарные и двойные кавычки, пробелы в путях и комментарии в конце строки.
Управляющие символы удаляются. Для рекурсивных подключений используется отдельный набор посещённых
файлов, поэтому циклические ссылки не вызывают бесконечную рекурсию.

Цель пользовательского bind-монтирования должна быть абсолютным путём и не может содержать `..`.
Если цель отсутствует, создаётся временная копия дерева от ближайшего существующего предка, не
являющегося корнем. Ошибка remount или регистрации отсоединяет монтирование и откатывает операцию.

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

[GPL-v3L icense](http://www.gnu.org/licenses/gpl.html)

## Звездная история

<a href="https://star-history.dera.page/#Tools-cx-app/meta-magic_mount-rs">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://star-history.dera.page/svg?repos=Tools-cx-app/meta-magic_mount-rs&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://star-history.dera.page/svg?repos=Tools-cx-app/meta-magic_mount-rs" />
   <img alt="Star History Chart" src="https://star-history.dera.page/svg?repos=Tools-cx-app/meta-magic_mount-rs" />
 </picture>
</a>
