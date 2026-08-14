use std::os::unix::fs::PermissionsExt;

use api::ConnectionInfo;

use super::{acquire_instance_lock, write_connection_file};

#[test]
fn instance_lock_rejects_a_second_daemon() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.lock");
    let first = acquire_instance_lock(&path).unwrap();

    assert!(acquire_instance_lock(&path).is_err());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap().trim(),
        std::process::id().to_string()
    );
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );

    drop(first);
    acquire_instance_lock(&path).unwrap();
}

#[tokio::test]
async fn connection_file_is_json_and_owner_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.json");
    let info = ConnectionInfo {
        port: 43127,
        token: "secret".into(),
    };

    write_connection_file(&path, &info).await.unwrap();

    let parsed: ConnectionInfo =
        serde_json::from_slice(&tokio::fs::read(&path).await.unwrap()).unwrap();
    assert_eq!(parsed, info);
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}
