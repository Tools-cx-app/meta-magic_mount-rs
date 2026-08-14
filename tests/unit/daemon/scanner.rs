use std::fs;

use super::*;

fn create_valid_prop_content(id: &str) -> String {
    format!("id={id}\nname=Test Module\nversion=1.0.0\nauthor=Tester\ndescription=A test module\n")
}

#[tokio::test]
async fn test_read_prop_success() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let prop_path = tmp_dir.path().join("module.prop");
    fs::write(&prop_path, "id=test\nname=test\nversion=v1.0\n").unwrap();
    let res = read_prop(&prop_path).await.unwrap();
    assert_eq!(res.get("id").unwrap(), "test");
    assert_eq!(res.get("name").unwrap(), "test");
    assert_eq!(res.get("version").unwrap(), "v1.0");
}

#[tokio::test]
async fn test_read_prop_file_not_found() {
    assert!(read_prop("non_existent_file.prop").await.is_err());
}

#[tokio::test]
async fn test_list_modules_empty_dir() {
    let tmp_dir = tempfile::tempdir().unwrap();
    assert!(list_modules(tmp_dir.path(), &[]).await.is_empty());
}

#[tokio::test]
async fn test_list_modules_integration() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let module_dir = tmp_dir.path();
    for (id, partition) in [
        ("test1", "system"),
        ("test2", "system"),
        ("test3", "system"),
        ("test6", "vendor"),
    ] {
        let module = module_dir.join(id);
        fs::create_dir_all(module.join(partition)).unwrap();
        fs::write(module.join("module.prop"), create_valid_prop_content(id)).unwrap();
    }
    let test2 = module_dir.join("test2");
    fs::File::create(test2.join(defs::DISABLE_FILE_NAME)).unwrap();
    let test3 = module_dir.join("test3");
    fs::File::create(test3.join(defs::SKIP_MOUNT_FILE_NAME)).unwrap();
    let test4 = module_dir.join("test4");
    fs::create_dir_all(&test4).unwrap();
    fs::write(test4.join("module.prop"), "id=test4\n").unwrap();
    let test5 = module_dir.join("test5");
    fs::create_dir_all(&test5).unwrap();
    fs::write(
        test5.join("module.prop"),
        create_valid_prop_content("test5"),
    )
    .unwrap();

    let result = list_modules(module_dir, &["vendor".to_string()]).await;
    assert_eq!(result.len(), 5);
    assert!(result[0].is_mounted);
    assert!(!result[1].is_mounted);
    assert!(!result[2].is_mounted);
    assert!(!result[3].is_mounted);
    assert!(result[4].is_mounted);
}
