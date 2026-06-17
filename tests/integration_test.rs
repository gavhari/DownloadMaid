use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_full_workflow() {
    let test_dir = PathBuf::from("/tmp/downloadmaid_test");
    
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir(&test_dir).unwrap();
    
    File::create(test_dir.join("doc.pdf")).unwrap().write_all(b"pdf").unwrap();
    File::create(test_dir.join("photo.jpg")).unwrap().write_all(b"jpg").unwrap();
    File::create(test_dir.join("archive.zip")).unwrap().write_all(b"zip").unwrap();
    File::create(test_dir.join("README")).unwrap().write_all(b"readme").unwrap();
    File::create(test_dir.join(".hidden")).unwrap().write_all(b"hidden").unwrap();
    fs::create_dir(test_dir.join("subdir")).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", test_dir.to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    assert!(test_dir.join("pdf/doc.pdf").exists());
    assert!(test_dir.join("jpg/photo.jpg").exists());
    assert!(test_dir.join("zip/archive.zip").exists());
    assert!(test_dir.join("others/README").exists());
    assert!(test_dir.join(".hidden").exists());
    assert!(test_dir.join("subdir").exists());
    
    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_dry_run_no_changes() {
    let test_dir = PathBuf::from("/tmp/downloadmaid_dry_test");
    
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir(&test_dir).unwrap();
    
    File::create(test_dir.join("test.pdf")).unwrap().write_all(b"pdf").unwrap();
    File::create(test_dir.join("photo.jpg")).unwrap().write_all(b"jpg").unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", test_dir.to_str().unwrap(), "--dry-run"])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[dry-run]"));
    
    assert!(test_dir.join("test.pdf").exists());
    assert!(test_dir.join("photo.jpg").exists());
    assert!(!test_dir.join("pdf").exists());
    assert!(!test_dir.join("jpg").exists());
    
    fs::remove_dir_all(&test_dir).unwrap();
}
