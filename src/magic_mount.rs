// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{
    collections::hash_map::Entry,
    fmt,
    fs::{self, DirEntry, FileType, Metadata, create_dir, create_dir_all, read_link},
    os::unix::fs::{FileTypeExt, MetadataExt, symlink},
    path::{Path, PathBuf},
    sync::atomic::AtomicU32,
};

use anyhow::Context;
use extattr::lgetxattr;
use rustc_hash::FxHashMap;
use rustix::{
    fs::{Gid, Mode, Uid, chmod, chown},
    mount::{
        MountFlags, MountPropagationFlags, mount, mount_bind, mount_change, mount_move,
        mount_remount,
    },
    path::Arg,
};

use crate::{
    defs,
    errors::{Error, Result},
    ksucalls::send_unmountable,
    mount_list,
    parser::COMMAND_LIST,
    utils::{ensure_dir_exists, lgetfilecon, lsetfilecon, validate_module_id},
};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum NodeFileType {
    RegularFile,
    Directory,
    Symlink,
    Whiteout,
}

impl From<FileType> for NodeFileType {
    fn from(value: FileType) -> Self {
        if value.is_file() {
            Self::RegularFile
        } else if value.is_dir() {
            Self::Directory
        } else if value.is_symlink() {
            Self::Symlink
        } else {
            Self::Whiteout
        }
    }
}

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub file_type: NodeFileType,
    pub children: FxHashMap<String, Self>,
    // the module that owned this node
    pub module_path: Option<PathBuf>,
    pub replace: bool,
    pub skip: bool,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_tree(f, 0)
    }
}

impl fmt::Display for NodeFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RegularFile => "RegularFile",
            Self::Directory => "Directory",
            Self::Symlink => "Symlink",
            Self::Whiteout => "Whiteout",
        })
    }
}

impl Node {
    fn debug_tree(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indent_str = "  ".repeat(indent);

        write!(f, "{}{} ({})", indent_str, self.name, self.file_type)?;
        if let Some(path) = &self.module_path {
            write!(f, " [{}]", path.display())?;
        }
        if self.replace {
            write!(f, " [REPLACE]")?;
        }
        if self.skip {
            write!(f, " [SKIP]")?;
        }
        writeln!(f)?;

        for child in self.children.values() {
            child.debug_tree(f, indent + 1)?;
        }
        Ok(())
    }
}

impl Node {
    pub fn collect_module_files<P>(&mut self, module_dir: P) -> Result<bool>
    where
        P: AsRef<Path>,
    {
        let dir = module_dir.as_ref();
        let mut has_file = false;
        for entry in dir.read_dir()?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            let node = match self.children.entry(name.clone()) {
                Entry::Occupied(o) => Some(o.into_mut()),
                Entry::Vacant(v) => Self::new_module(&name, &entry).map(|it| v.insert(it)),
            };

            if let Some(node) = node {
                has_file |= if node.file_type == NodeFileType::Directory {
                    node.collect_module_files(dir.join(&node.name))? || node.replace
                } else {
                    true
                }
            }
        }

        Ok(has_file)
    }

    fn dir_is_replace<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        if lgetxattr(&path, defs::REPLACE_DIR_XATTR)
            .is_ok_and(|s| String::from_utf8_lossy(&s) == "y")
        {
            true
        } else {
            path.as_ref().join(defs::REPLACE_DIR_FILE_NAME).exists()
        }
    }

    fn dir_is_skip<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let list = COMMAND_LIST.get().unwrap();
        let path = path.as_ref().to_string_lossy();
        list.iter()
            .any(|s| matches!(s, crate::parser::MountType::Ignore { source } if source == &path))
            || path.ends_with(".replace")
    }

    pub fn new_root<S>(name: S) -> Self
    where
        S: AsRef<str> + Into<String>,
    {
        Self {
            name: name.into(),
            file_type: NodeFileType::Directory,
            children: FxHashMap::default(),
            module_path: None,
            replace: false,
            skip: false,
        }
    }

    pub fn new_module<S>(name: &S, entry: &DirEntry) -> Option<Self>
    where
        S: ToString,
    {
        if let Ok(metadata) = entry.metadata() {
            let path = entry.path();
            let file_type = if metadata.file_type().is_char_device() && metadata.rdev() == 0 {
                NodeFileType::Whiteout
            } else {
                NodeFileType::from(metadata.file_type())
            };

            let replace = file_type == NodeFileType::Directory && Self::dir_is_replace(&path);
            let skip = Self::dir_is_skip(&path);
            if replace {
                log::debug!("{} need replace", path.display());
            }
            if skip {
                log::debug!("{} was skip", path.display());
            }
            return Some(Self {
                name: name.to_string(),
                file_type,
                children: FxHashMap::default(),
                module_path: Some(path),
                replace,
                skip,
            });
        }

        None
    }
}
fn metadata_path<P>(path: P, node: &Node) -> Result<(Metadata, PathBuf)>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.exists() {
        Ok((path.metadata()?, path.to_path_buf()))
    } else if let Some(module_path) = &node.module_path {
        Ok((module_path.metadata()?, module_path.clone()))
    } else {
        Err(Error::MountRootFile {
            path: path.display().to_string(),
        })
    }
}

pub fn tmpfs_skeleton<P>(path: P, work_dir_path: P, node: &Node) -> Result<()>
where
    P: AsRef<Path>,
{
    let (path, work_dir_path) = (path.as_ref(), work_dir_path.as_ref());
    log::debug!(
        "creating tmpfs skeleton for {} at {}",
        path.display(),
        work_dir_path.display()
    );

    create_dir_all(work_dir_path)?;

    let (metadata, path) = metadata_path(path, node)?;

    chmod(work_dir_path, Mode::from_raw_mode(metadata.mode()))?;
    chown(
        work_dir_path,
        Some(Uid::from_raw(metadata.uid())),
        Some(Gid::from_raw(metadata.gid())),
    )?;
    lsetfilecon(work_dir_path, lgetfilecon(path)?.as_str())?;

    Ok(())
}

pub fn mount_mirror<P>(path: P, work_dir_path: P, entry: &DirEntry) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().join(entry.file_name());
    let work_dir_path = work_dir_path.as_ref().join(entry.file_name());
    let file_type = entry.file_type()?;

    if file_type.is_file() {
        log::debug!(
            "mount mirror file {} -> {}",
            path.display(),
            work_dir_path.display()
        );
        fs::File::create(&work_dir_path)?;
        mount_bind(&path, &work_dir_path)?;
    } else if file_type.is_dir() {
        log::debug!(
            "mount mirror dir {} -> {}",
            path.display(),
            work_dir_path.display()
        );
        create_dir(&work_dir_path)?;
        let metadata = entry.metadata()?;
        chmod(&work_dir_path, Mode::from_raw_mode(metadata.mode()))?;
        chown(
            &work_dir_path,
            Some(Uid::from_raw(metadata.uid())),
            Some(Gid::from_raw(metadata.gid())),
        )?;
        lsetfilecon(&work_dir_path, lgetfilecon(&path)?.as_str())?;
        for entry in path.read_dir()?.flatten() {
            mount_mirror(&path, &work_dir_path, &entry)?;
        }
    } else if file_type.is_symlink() {
        log::debug!(
            "create mirror symlink {} -> {}",
            path.display(),
            work_dir_path.display()
        );
        clone_symlink(&path, &work_dir_path)?;
    }

    Ok(())
}

pub fn collect_module_files(
    module_dir: &Path,
    extra_partitions: &[String],
) -> Result<Option<Node>> {
    let mut root = Node::new_root("");
    let mut system = Node::new_root("system");
    let module_root = module_dir;
    let mut has_file = false;

    log::debug!("begin collect module files: {}", module_root.display());

    for entry in module_root.read_dir()?.flatten() {
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let id = entry.file_name().to_str().unwrap().to_string();
        log::debug!("processing new module: {id}");

        let prop = entry.path().join("module.prop");
        if !prop.exists() {
            log::debug!("skipped module {id}, because not found module.prop");
            continue;
        }
        let string = fs::read_to_string(prop)?;
        for line in string.lines() {
            if line.starts_with("id")
                && let Some((_, value)) = line.split_once('=')
            {
                validate_module_id(value)?;
            }
        }

        if entry.path().join(defs::DISABLE_FILE_NAME).exists()
            || entry.path().join(defs::REMOVE_FILE_NAME).exists()
            || entry.path().join(defs::SKIP_MOUNT_FILE_NAME).exists()
        {
            log::debug!("skipped module {id}, due to disable/remove/skip_mount");
            continue;
        }

        let mod_system = entry.path().join("system");
        if !mod_system.is_dir() {
            log::debug!("{id} due not modify system");
            continue;
        }

        log::debug!("collecting {}", entry.path().display());

        has_file |= system.collect_module_files(&mod_system)?;
    }

    if has_file {
        const BUILTIN_PARTITIONS: [(&str, bool); 4] = [
            ("vendor", true),
            ("system_ext", true),
            ("product", true),
            ("odm", false),
        ];

        for (partition, require_symlink) in BUILTIN_PARTITIONS {
            let path_of_root = Path::new("/").join(partition);
            let path_of_system = Path::new("/system").join(partition);
            if path_of_root.is_dir() && (!require_symlink || path_of_system.is_symlink()) {
                let name = partition.to_string();
                if let Some(node) = system.children.remove(&name) {
                    root.children.insert(name, node);
                }
            }
        }

        for partition in extra_partitions {
            if BUILTIN_PARTITIONS.iter().any(|(p, _)| p == partition) {
                continue;
            }
            if partition == "system" {
                continue;
            }

            let path_of_root = Path::new("/").join(partition);
            let path_of_system = Path::new("/system").join(partition);
            let require_symlink = false;

            if path_of_root.is_dir() && (!require_symlink || path_of_system.is_symlink()) {
                let name = partition.clone();
                if let Some(node) = system.children.remove(&name) {
                    log::debug!("attach extra partition '{name}' to root");
                    root.children.insert(name, node);
                }
            }
        }

        root.children.insert("system".to_string(), system);
        Ok(Some(root))
    } else {
        Ok(None)
    }
}

pub fn clone_symlink<S>(src: S, dst: S) -> Result<()>
where
    S: AsRef<Path>,
{
    let src_symlink = read_link(src.as_ref())?;
    symlink(&src_symlink, dst.as_ref())?;
    lsetfilecon(dst.as_ref(), lgetfilecon(src.as_ref())?.as_str())?;
    log::debug!(
        "clone symlink {} -> {}({})",
        dst.as_ref().display(),
        dst.as_ref().display(),
        src_symlink.display()
    );
    Ok(())
}
static MOUNTDED_FILES: AtomicU32 = AtomicU32::new(0);
static IGNORED_FILES: AtomicU32 = AtomicU32::new(0);
static MOUNTDED_SYMBOLS_FILES: AtomicU32 = AtomicU32::new(0);

struct MagicMount<'a> {
    node: Node,
    path: PathBuf,
    work_dir_path: PathBuf,
    has_tmpfs: bool,
    umount: bool,
    mounts: &'a mount_list::MountList,
}

impl<'a> MagicMount<'a> {
    fn new<P>(
        node: &Node,
        path: P,
        work_dir_path: P,
        has_tmpfs: bool,
        umount: bool,
        mounts: &'a mount_list::MountList,
    ) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            node: node.clone(),
            path: path.as_ref().join(node.name.clone()),
            work_dir_path: work_dir_path.as_ref().join(node.name.clone()),
            has_tmpfs,
            umount,
            mounts,
        }
    }

    fn do_mount(&mut self) -> Result<()> {
        match self.node.file_type {
            NodeFileType::Symlink => self.symlink(),
            NodeFileType::RegularFile => self.regular_file(),
            NodeFileType::Directory => self.directory(),
            NodeFileType::Whiteout => {
                log::debug!("file {} is removed", self.path.display());
                Ok(())
            }
        }
    }
}

impl MagicMount<'_> {
    fn symlink(&self) -> Result<()> {
        if let Some(module_path) = &self.node.module_path {
            log::debug!(
                "create module symlink {} -> {}",
                module_path.display(),
                self.work_dir_path.display()
            );
            clone_symlink(module_path, &self.work_dir_path).with_context(|| {
                format!(
                    "create module symlink {} -> {}",
                    module_path.display(),
                    self.work_dir_path.display(),
                )
            })?;
            let mounted = MOUNTDED_SYMBOLS_FILES.load(std::sync::atomic::Ordering::Relaxed) + 1;
            MOUNTDED_SYMBOLS_FILES.store(mounted, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        } else {
            Err(Error::MountRootSymlink {
                path: self.path.display().to_string(),
            })
        }
    }

    fn regular_file(&self) -> Result<()> {
        let target = if self.has_tmpfs {
            fs::File::create(&self.work_dir_path)?;
            &self.work_dir_path
        } else {
            &self.path
        };

        if self.node.module_path.is_none() {
            return Err(Error::MountRootFile {
                path: self.path.display().to_string(),
            });
        }

        let module_path = &self.node.module_path.clone().unwrap();

        log::debug!(
            "mount module file {} -> {}",
            module_path.display(),
            self.work_dir_path.display()
        );

        if mount_bind(module_path, target)
            .with_context(|| {
                format!(
                    "mount module file {} -> {}",
                    module_path.display(),
                    self.work_dir_path.display(),
                )
            })
            .is_ok()
        {
            self.mounts.record_if_final(&self.path, self.has_tmpfs);
            if self.umount && !self.work_dir_path.starts_with("/mnt") {
                send_unmountable(target);
            }
        }

        // we should use MS_REMOUNT | MS_BIND | MS_xxx to change mount flags
        if let Err(e) = mount_remount(target, MountFlags::RDONLY | MountFlags::BIND, "") {
            log::warn!("make file {} ro: {e:#?}", target.display());
        }

        let mounted = MOUNTDED_FILES.load(std::sync::atomic::Ordering::Relaxed) + 1;
        MOUNTDED_FILES.store(mounted, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn directory(&mut self) -> Result<()> {
        let mut tmpfs = !self.has_tmpfs && self.node.replace && self.node.module_path.is_some();

        if !self.has_tmpfs && !tmpfs {
            for it in &mut self.node.children {
                let (name, node) = it;
                let real_path = self.path.join(name);
                let need = match node.file_type {
                    NodeFileType::Symlink => true,
                    NodeFileType::Whiteout => real_path.exists(),
                    _ => {
                        if let Ok(metadata) = real_path.symlink_metadata() {
                            let file_type = NodeFileType::from(metadata.file_type());
                            file_type != node.file_type || file_type == NodeFileType::Symlink
                        } else {
                            // real path not exists
                            true
                        }
                    }
                };
                if need {
                    if self.node.module_path.is_none() {
                        log::error!(
                            "cannot create tmpfs on {}, ignore: {name}",
                            self.path.display()
                        );
                        let ignored_files =
                            IGNORED_FILES.load(std::sync::atomic::Ordering::Relaxed) + 1;
                        IGNORED_FILES.store(ignored_files, std::sync::atomic::Ordering::Relaxed);
                        node.skip = true;
                        continue;
                    }
                    tmpfs = true;
                    break;
                }
            }
        }
        let has_tmpfs = tmpfs || self.has_tmpfs;

        if has_tmpfs {
            tmpfs_skeleton(&self.path, &self.work_dir_path, &self.node)?;
        }

        if tmpfs {
            mount_bind(&self.work_dir_path, &self.work_dir_path).with_context(|| {
                format!(
                    "creating tmpfs for {} at {}",
                    self.path.display(),
                    self.work_dir_path.display(),
                )
            })?;
        }

        if self.path.exists() && !self.node.replace {
            self.mount_path(has_tmpfs)?;
        }

        if self.node.replace {
            if self.node.module_path.is_none() {
                return Err(Error::DirDeclared {
                    path: self.path.display().to_string(),
                });
            }

            log::debug!("dir {} is replaced", self.path.display());
        }

        for (name, node) in &self.node.children {
            if node.skip {
                continue;
            }

            if let Err(e) = {
                Self::new(
                    node,
                    &self.path,
                    &self.work_dir_path,
                    has_tmpfs,
                    self.umount,
                    self.mounts,
                )
                .do_mount()
            }
            .with_context(|| format!("magic mount {}/{name}", self.path.display()))
            {
                if has_tmpfs {
                    return Err(e.into());
                }

                log::error!("mount child {}/{name} failed: {e:#?}", self.path.display());
            }
        }

        if tmpfs {
            log::debug!(
                "moving tmpfs {} -> {}",
                self.work_dir_path.display(),
                self.path.display()
            );

            if let Err(e) = mount_remount(
                &self.work_dir_path,
                MountFlags::RDONLY | MountFlags::BIND,
                "",
            ) {
                log::warn!("make dir {} ro: {e:#?}", self.path.display());
            }
            mount_move(&self.work_dir_path, &self.path).with_context(|| {
                format!(
                    "moving tmpfs {} -> {}",
                    self.work_dir_path.display(),
                    self.path.display()
                )
            })?;
            self.mounts.commit_staged_under(&self.path);
            self.mounts.record(&self.path);
            // make private to reduce peer group count
            if let Err(e) = mount_change(
                &self.path,
                MountPropagationFlags::PRIVATE | MountPropagationFlags::REC,
            ) {
                log::warn!("make dir {} private: {e:#?}", self.path.display());
            }

            if self.umount {
                // tell ksu about this one too
                send_unmountable(&self.path);
            }
        }
        Ok(())
    }
}

impl MagicMount<'_> {
    fn mount_path(&mut self, has_tmpfs: bool) -> Result<()> {
        for entry in self.path.read_dir()?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let result = {
                if let Some(node) = self.node.children.remove(&name) {
                    if node.skip {
                        continue;
                    }

                    Self::new(
                        &node,
                        &self.path,
                        &self.work_dir_path,
                        has_tmpfs,
                        self.umount,
                        self.mounts,
                    )
                    .do_mount()
                    .with_context(|| format!("magic mount {}/{name}", self.path.display()))
                } else if has_tmpfs {
                    mount_mirror(&self.path, &self.work_dir_path, &entry)
                        .with_context(|| format!("mount mirror {}/{name}", self.path.display()))
                } else {
                    Ok(())
                }
            };

            if let Err(e) = result {
                if has_tmpfs {
                    return Err(e.into());
                }
                log::error!("mount child {}/{name} failed: {e:#?}", self.path.display());
            }
        }

        Ok(())
    }
}

pub fn magic_mount<P>(
    module_dir: P,
    mount_source: &str,
    extra_partitions: &[String],
    umount: bool,
    mounts: &mount_list::MountList,
) -> Result<()>
where
    P: AsRef<Path>,
{
    if let Some(root) = collect_module_files(module_dir.as_ref(), extra_partitions)? {
        log::debug!("collected: {root:?}");
        let tmp_root = Path::new("/debug_ramdisk");
        let tmp_dir = tmp_root.join("workdir");
        ensure_dir_exists(&tmp_dir)?;

        mount(mount_source, &tmp_dir, "tmpfs", MountFlags::empty(), None).context("mount tmp")?;
        mount_change(
            &tmp_dir,
            MountPropagationFlags::PRIVATE | MountPropagationFlags::REC,
        )
        .context("make tmp recursively private")?;

        MagicMount::new(
            &root,
            Path::new("/"),
            tmp_dir.as_path(),
            false,
            umount,
            mounts,
        )
        .do_mount()?;
    } else {
        log::info!("no modules to mount, skipping!");
    }
    let mounted_symbols = MOUNTDED_SYMBOLS_FILES.load(std::sync::atomic::Ordering::Relaxed);
    let mounted_files = MOUNTDED_FILES.load(std::sync::atomic::Ordering::Relaxed);
    log::info!("mounted files: {mounted_files}, mounted symlinks: {mounted_symbols}");
    crate::utils::update_desc(mounted_files, mounted_symbols)?;
    Ok(())
}
