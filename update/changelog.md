## 2.2.3 - 2026-02-06

**[View changes](https://github.com/Tools-cx-app/meta-magic_mount/compare/f6f60e6cd3d0e0417cf716cd42f7d7d2a2a20811...51e1e14e42ab211194fd7b4bd35dd9749487dbec)**(f6f60e6...51e1e14)

### <!-- 0 --> Features

- Only support ksu/ap([ceb2d4a](https://github.com/Tools-cx-app/meta-magic_mount/commit/ceb2d4a33fe0887d54a1b4cd3163d61a4a694220))
- Automatic exit when failed to mount tmpfs on tmpfsdir([57b6473](https://github.com/Tools-cx-app/meta-magic_mount/commit/57b64735c6047d50dfe6635144e948ba84ae83de))
- Removed `/debug_ramdisk` in default try-umount list([d9beda3](https://github.com/Tools-cx-app/meta-magic_mount/commit/d9beda3f1359d7cb12dd6f92ceac2776bda6b916))
- Support random path of tempdir([34f91f1](https://github.com/Tools-cx-app/meta-magic_mount/commit/34f91f10473e892f6341e38a85c2d4a73b92f7a6))

### <!-- 1 --> Bug Fixes

- Improve layout and word breaking in module display (#70)([fa3eeae](https://github.com/Tools-cx-app/meta-magic_mount/commit/fa3eeaed815e0cb8d9d08c34e0f9331bb9acb35c))
- Fix complete in xtask([c5b8990](https://github.com/Tools-cx-app/meta-magic_mount/commit/c5b8990d4baa5a7faa677d2bea67a72d8e76ade9))
- Fix rz panic([2f9310e](https://github.com/Tools-cx-app/meta-magic_mount/commit/2f9310e478b149ac325ee316458e68b0c1aa9762))
- Fixed parameters across multiple platforms([633107f](https://github.com/Tools-cx-app/meta-magic_mount/commit/633107f658a026c16099854cb2ab58242ad9f700))
- Fix Correct name([493fbba](https://github.com/Tools-cx-app/meta-magic_mount/commit/493fbbaf1ede31581bf76534917a3845e0e54433))

### <!-- 3 --> Refactor

- Remove hymofs, diagnostics and custom tempdir support([95408bd](https://github.com/Tools-cx-app/meta-magic_mount/commit/95408bd0041e3879bbf35bf8571ef2a774b1081b))
- Migrate to KernelSU theme system([1c8791d](https://github.com/Tools-cx-app/meta-magic_mount/commit/1c8791d6102043d2859c1c90ddd688bce88df316))

### <!-- 4 --> Documentation

- Moved README to docs([48ea0d1](https://github.com/Tools-cx-app/meta-magic_mount/commit/48ea0d14eb6b139ab4a9199852f3eee9bbe0ea85))

### <!-- 5 --> Performance

- Determine the ZN state directly using the ZN path.([4933cc6](https://github.com/Tools-cx-app/meta-magic_mount/commit/4933cc6199c6a427afc39d74bbedd4fdfb3111ff))

### <!-- 8 --> Miscellaneous Tasks

- Passed webui lints check([6774aa7](https://github.com/Tools-cx-app/meta-magic_mount/commit/6774aa74589fa3e7c59e970975042f3379950224))
- Resolving the issue #74([72e378a](https://github.com/Tools-cx-app/meta-magic_mount/commit/72e378a6a0235de837a53a8ef7030ec95685bedc))
- Fix cache error && dropped rust cache in release([4b8be3e](https://github.com/Tools-cx-app/meta-magic_mount/commit/4b8be3eb3188ffafc8f278da51643d5d223ac8bb))
- Picked #74 suggestions([2c905bd](https://github.com/Tools-cx-app/meta-magic_mount/commit/2c905bd045e9500928f8a40935998eee56ac04fb))
- Add run number in bot message([ff9239b](https://github.com/Tools-cx-app/meta-magic_mount/commit/ff9239beff4dc4d41f1117ec5a0dbc301a3e7c19))
- Fmt && make cargo clippy happy([9b4627e](https://github.com/Tools-cx-app/meta-magic_mount/commit/9b4627e1600343b73a0d35998a45e6729b6ef1fa))


