
pub(super) const REPLACE_DIR_FILE_NAME: &str = ".replace";
pub(super) const REPLACE_DIR_XATTR: &str = "trusted.overlay.opaque";

use std::{
    collections::{HashMap, hash_map::Entry},
    ffi::CString,
    fmt,
    fs::{self, DirEntry, FileType, create_dir, create_dir_all, read_dir, read_link},
    os::unix::fs::{FileTypeExt, MetadataExt, symlink},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use extattr::lgetxattr;
use rustix::{
    fs::{Gid, Mode, Uid, chmod, chown},
    mount::{
        MountFlags, MountPropagationFlags, UnmountFlags, mount, mount_bind, mount_change,
        mount_move, mount_remount, unmount,
    },
};

use crate::utils::{ensure_dir_exists, lgetfilecon, lsetfilecon, send_unmountable};


#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(super) enum NodeFileType {
    RegularFile,
    Directory,
    Symlink,
    Whiteout,
}

impl NodeFileType {
    pub(super) fn from_file_type(file_type: FileType) -> Option<Self> {
        if file_type.is_file() {
            Some(Self::RegularFile)
        } else if file_type.is_dir() {
            Some(Self::Directory)
        } else if file_type.is_symlink() {
            Some(Self::Symlink)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(super) struct Node {
    pub(super) name: String,
    pub(super) file_type: NodeFileType,
    pub(super) children: HashMap<String, Node>,
    pub(super) module_path: Option<PathBuf>,
    pub(super) replace: bool,
    pub(super) skip: bool,
}

impl fmt::Display for NodeFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory => write!(f, "DIR"),
            Self::RegularFile => write!(f, "FILE"),
            Self::Symlink => write!(f, "LINK"),
            Self::Whiteout => write!(f, "WHT"),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_tree(
            node: &Node, 
            f: &mut fmt::Formatter<'_>, 
            prefix: &str, 
            is_last: bool, 
            is_root: bool
        ) -> fmt::Result {
            let connector = if is_root { "" } else if is_last { "└── " } else { "├── " };
            
            let name = if node.name.is_empty() { "/" } else { &node.name };
            
            let mut flags = Vec::new();
            if node.replace { flags.push("REPLACE"); }
            if node.skip { flags.push("SKIP"); }
            
            let flag_str = if flags.is_empty() { 
                String::new() 
            } else { 
                format!(" [{}]", flags.join("|")) 
            };

            let source_str = if let Some(p) = &node.module_path {
                format!(" -> {}", p.display())
            } else {
                String::new()
            };

            writeln!(f, "{}{}{} [{}]{}{}", prefix, connector, name, node.file_type, flag_str, source_str)?;

            let child_prefix = if is_root {
                ""
            } else if is_last {
                "    "
            } else {
                "│   "
            };
            let new_prefix = format!("{}{}", prefix, child_prefix);

            let mut children: Vec<_> = node.children.values().collect();
            children.sort_by(|a, b| a.name.cmp(&b.name));

            for (i, child) in children.iter().enumerate() {
                let is_last_child = i == children.len() - 1;
                print_tree(child, f, &new_prefix, is_last_child, false)?;
            }
            Ok(())
        }

        print_tree(self, f, "", true, true)
    }
}

impl Node {
    pub(super) fn collect_module_files<T: AsRef<Path>>(&mut self, module_dir: T) -> Result<bool> {
        let dir = module_dir.as_ref();
        let mut has_file = false;
        
        if let Ok(entries) = dir.read_dir() {
            for entry in entries.flatten() {
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
        }

        Ok(has_file)
    }

    pub(super) fn dir_is_replace<P>(path: P) -> Result<bool>
    where
        P: AsRef<Path>,
    {
        if let Ok(v) = lgetxattr(&path, REPLACE_DIR_XATTR) {
            if String::from_utf8_lossy(&v) == "y" {
                return Ok(true);
            }
        }

        let path_str = path.as_ref().to_string_lossy();
        let c_path = CString::new(path_str.as_bytes())?;
        
        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY) };

        if fd < 0 {
            return Ok(false);
        }

        let exists = unsafe {
            let replace = CString::new(REPLACE_DIR_FILE_NAME)?;
            let ret = libc::faccessat(fd, replace.as_ptr(), libc::F_OK, 0);
            libc::close(fd);
            ret
        };

        if exists == 0 { Ok(true) } else { Ok(false) }
    }

    pub(super) fn new_root<T: ToString>(name: T) -> Self {
        Node {
            name: name.to_string(),
            file_type: NodeFileType::Directory,
            children: Default::default(),
            module_path: None,
            replace: false,
            skip: false,
        }
    }

    pub(super) fn new_module<T: ToString>(name: T, entry: &DirEntry) -> Option<Self> {
        if let Ok(metadata) = entry.metadata() {
            let path = entry.path();
            let file_type = if metadata.file_type().is_char_device() && metadata.rdev() == 0 {
                Some(NodeFileType::Whiteout)
            } else {
                NodeFileType::from_file_type(metadata.file_type())
            };
            if let Some(file_type) = file_type {
                let mut replace = false;
                if file_type == NodeFileType::Directory {
                    if let Ok(s) = Self::dir_is_replace(&path) {
                         if s {
                            replace = true;
                         }
                    }
                }
                return Some(Node {
                    name: name.to_string(),
                    file_type,
                    children: Default::default(),
                    module_path: Some(path),
                    replace,
                    skip: false,
                });
            }
        }

        None
    }
}


fn collect_module_files(content_paths: &[PathBuf], extra_partitions: &[String]) -> Result<Option<Node>> {
    let mut root = Node::new_root("");
    let mut system = Node::new_root("system");
    let mut has_file = false;

    for module_path in content_paths {
        let module_system = module_path.join("system");
        if !module_system.is_dir() {
            continue;
        }

        log::debug!("collecting {}", module_path.display());
        has_file |= system.collect_module_files(&module_system)?;
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
                if let Some(mut node) = system.children.remove(&name) {
                    if node.file_type == NodeFileType::Symlink {
                        if let Some(ref p) = node.module_path {
                            if let Ok(meta) = fs::metadata(p) {
                                if meta.is_dir() {
                                    log::debug!("treating symlink {} as directory for recursion", name);
                                    node.file_type = NodeFileType::Directory;
                                }
                            }
                        }
                    }
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
                let name = partition.to_string();
                if let Some(mut node) = system.children.remove(&name) {
                    log::debug!("attach extra partition '{}' to root", name);
                    if node.file_type == NodeFileType::Symlink {
                        if let Some(ref p) = node.module_path {
                            if let Ok(meta) = fs::metadata(p) {
                                if meta.is_dir() {
                                    log::debug!("treating symlink {} as directory for recursion", name);
                                    node.file_type = NodeFileType::Directory;
                                }
                            }
                        }
                    }
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

fn clone_symlink<Src: AsRef<Path>, Dst: AsRef<Path>>(src: Src, dst: Dst) -> Result<()> {
    let src_symlink = read_link(src.as_ref())?;
    symlink(&src_symlink, dst.as_ref())?;
    lsetfilecon(dst.as_ref(), lgetfilecon(src.as_ref())?.as_str())?;
    Ok(())
}

fn mount_mirror<P: AsRef<Path>, WP: AsRef<Path>>(path: P, work_dir_path: WP, entry: &DirEntry) -> Result<()> {
    let path = path.as_ref().join(entry.file_name());
    let work_dir_path = work_dir_path.as_ref().join(entry.file_name());
    if entry.file_type()?.is_file() {
        fs::File::create(&work_dir_path)?;
        mount_bind(&path, &work_dir_path)?;
    } else if entry.file_type()?.is_dir() {
        create_dir(&work_dir_path)?;
        let metadata = entry.metadata()?;
        chmod(&work_dir_path, Mode::from_raw_mode(metadata.mode()))?;
        unsafe {
            chown(&work_dir_path, Some(Uid::from_raw(metadata.uid())), Some(Gid::from_raw(metadata.gid())))?;
        }
        lsetfilecon(&work_dir_path, lgetfilecon(&path)?.as_str())?;
        for entry in read_dir(&path)?.flatten() {
            mount_mirror(&path, &work_dir_path, &entry)?;
        }
    } else if entry.file_type()?.is_symlink() {
        clone_symlink(&path, &work_dir_path)?;
    }
    Ok(())
}

fn mount_file<P: AsRef<Path>, WP: AsRef<Path>>(
    path: P,
    work_dir_path: WP,
    node: &Node,
    has_tmpfs: bool,
    disable_umount: bool
) -> Result<()> {
    let target_path = if has_tmpfs {
        fs::File::create(&work_dir_path)?;
        work_dir_path.as_ref()
    } else {
        path.as_ref()
    };
    
    if let Some(module_path) = &node.module_path {
        mount_bind(module_path, target_path)?;
        if !disable_umount {
            let _ = send_unmountable(target_path);
        }
        let _ = mount_remount(target_path, MountFlags::RDONLY | MountFlags::BIND, "");
    }
    Ok(())
}

fn mount_symlink<WP: AsRef<Path>>(
    work_dir_path: WP,
    node: &Node,
) -> Result<()> {
    if let Some(module_path) = &node.module_path {
        clone_symlink(module_path, work_dir_path)?;
    }
    Ok(())
}

fn should_create_tmpfs(node: &Node, path: &Path, has_tmpfs: bool) -> bool {
    if has_tmpfs { return true; }
    
    if node.replace && node.module_path.is_some() { return true; }

    for (name, child) in &node.children {
        let real_path = path.join(name);
        
        let need = match child.file_type {
            NodeFileType::Symlink => true,
            NodeFileType::Whiteout => real_path.exists(),
            _ => {
                if let Ok(meta) = real_path.symlink_metadata() {
                    let ft = NodeFileType::from_file_type(meta.file_type()).unwrap_or(NodeFileType::Whiteout);
                    ft != child.file_type || ft == NodeFileType::Symlink
                } else { 
                    true
                }
            }
        };

        if need {
            if node.module_path.is_none() {
                log::error!(
                    "Cannot create tmpfs on {} (no module source), ignoring conflicting child: {}",
                    path.display(),
                    name
                );
                return false;
            }
            return true;
        }
    }
    
    false
}

fn prepare_tmpfs_dir<P: AsRef<Path>, WP: AsRef<Path>>(
    path: P,
    work_dir_path: WP,
    node: &Node,
) -> Result<()> {
    create_dir_all(work_dir_path.as_ref())?;
    
    let (metadata, src_path) = if path.as_ref().exists() { 
        (path.as_ref().metadata()?, path.as_ref()) 
    } else { 
        let mp = node.module_path.as_ref().unwrap();
        (mp.metadata()?, mp.as_path())
    };

    chmod(work_dir_path.as_ref(), Mode::from_raw_mode(metadata.mode()))?;
    unsafe {
        chown(work_dir_path.as_ref(), Some(Uid::from_raw(metadata.uid())), Some(Gid::from_raw(metadata.gid())))?;
    }
    lsetfilecon(work_dir_path.as_ref(), lgetfilecon(src_path)?.as_str())?;
    
    mount_bind(work_dir_path.as_ref(), work_dir_path.as_ref())?;
    Ok(())
}

fn mount_directory_children<P: AsRef<Path>, WP: AsRef<Path>>(
    path: P,
    work_dir_path: WP,
    node: Node,
    has_tmpfs: bool,
    disable_umount: bool,
) -> Result<()> {
    if has_tmpfs && path.as_ref().exists() && !node.replace {
        for entry in path.as_ref().read_dir()?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !node.children.contains_key(&name) {
                mount_mirror(&path, &work_dir_path, &entry)?;
            }
        }
    }

    for (_name, child_node) in node.children {
        if child_node.skip { continue; }

        do_magic_mount(
            &path, 
            &work_dir_path, 
            child_node, 
            has_tmpfs, 
            disable_umount
        )?;
    }
    Ok(())
}

fn finalize_tmpfs_overlay<P: AsRef<Path>, WP: AsRef<Path>>(
    path: P,
    work_dir_path: WP,
    disable_umount: bool,
) -> Result<()> {
    let _ = mount_remount(work_dir_path.as_ref(), MountFlags::RDONLY | MountFlags::BIND, "");
    mount_move(work_dir_path.as_ref(), path.as_ref())?;
    let _ = mount_change(path.as_ref(), MountPropagationFlags::PRIVATE);
    
    if !disable_umount {
        let _ = send_unmountable(path.as_ref());
    }
    Ok(())
}

fn do_magic_mount<P: AsRef<Path>, WP: AsRef<Path>>(
    path: P,
    work_dir_path: WP,
    current: Node,
    has_tmpfs: bool,
    disable_umount: bool,
) -> Result<()> {
    let name = current.name.clone();
    let path = path.as_ref().join(&name);
    let work_dir_path = work_dir_path.as_ref().join(&name);
    
    match current.file_type {
        NodeFileType::RegularFile => {
            mount_file(&path, &work_dir_path, &current, has_tmpfs, disable_umount)?;
        }
        NodeFileType::Symlink => {
            mount_symlink(&work_dir_path, &current)?;
        }
        NodeFileType::Directory => {
            let create_tmpfs = !has_tmpfs && should_create_tmpfs(&current, &path, false);
            let effective_tmpfs = has_tmpfs || create_tmpfs;

            if effective_tmpfs {
                if create_tmpfs {
                    prepare_tmpfs_dir(&path, &work_dir_path, &current)?;
                } else if has_tmpfs {
                    if !work_dir_path.exists() {
                        create_dir(&work_dir_path)?;
                        let (metadata, src_path) = if path.exists() { (path.metadata()?, &path) } 
                                                   else { (current.module_path.as_ref().unwrap().metadata()?, current.module_path.as_ref().unwrap()) };
                        chmod(&work_dir_path, Mode::from_raw_mode(metadata.mode()))?;
                        unsafe {
                            chown(&work_dir_path, Some(Uid::from_raw(metadata.uid())), Some(Gid::from_raw(metadata.gid())))?;
                        }
                        lsetfilecon(&work_dir_path, lgetfilecon(src_path)?.as_str())?;
                    }
                }
            }

            mount_directory_children(
                &path, 
                &work_dir_path, 
                current, 
                effective_tmpfs, 
                disable_umount
            )?;

            if create_tmpfs {
                finalize_tmpfs_overlay(&path, &work_dir_path, disable_umount)?;
            }
        }
        NodeFileType::Whiteout => {}
    }
    Ok(())
}

pub fn mount_partitions(
    tmp_path: &Path,
    module_paths: &[PathBuf],
    mount_source: &str,
    extra_partitions: &[String],
    disable_umount: bool,
) -> Result<()> {
    if let Some(root) = collect_module_files(module_paths, extra_partitions)? {
        log::debug!("Magic Mount Root: {}", root);

        let tmp_dir = tmp_path.join("workdir");
        ensure_dir_exists(&tmp_dir)?;

        mount(mount_source, &tmp_dir, "tmpfs", MountFlags::empty(), "").context("mount tmp")?;
        mount_change(&tmp_dir, MountPropagationFlags::PRIVATE).context("make tmp private")?;

        let result = do_magic_mount("/", &tmp_dir, root, false, disable_umount);

        if let Err(e) = unmount(&tmp_dir, UnmountFlags::DETACH) {
            log::error!("failed to unmount tmp {}", e);
        }
        fs::remove_dir(tmp_dir).ok();

        result
    } else {
        log::info!("No files to magic mount");
        Ok(())
    }
}
