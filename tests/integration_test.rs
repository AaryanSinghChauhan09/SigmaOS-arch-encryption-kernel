// Integration test for SigmaOS-arch-encryption-kernel
// Verifies the crate builds and links as a dependency.

use sigmaos_arch_encryption_kernel::{Algorithm, EncryptionKey, EncryptionManager};

#[test]
fn crate_links_and_runs() {
    // Arrange
    let mut mgr = EncryptionManager::new();
    let material = mgr.generate_key_material(Algorithm::AES256, 42);
    let key = EncryptionKey::new("key1", Algorithm::AES256, material);
    mgr.add_key(key).unwrap();

    // Act
    let res = mgr.encrypt("key1", b"test");

    // Assert
    assert!(res.is_ok());
    assert_eq!(mgr.list_keys().len(), 1);
}