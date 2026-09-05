use rouxflow_bluetoothcube::codec::{BitView, parse_mac_salt, derive_gan_keys, gan_encrypt, gan_decrypt};

#[test]
fn test_bitview_basic() {
    // 0xAB = 10101011, 0xCD = 11001101
    let data = [0xAB, 0xCD];
    let view = BitView::new(&data);

    // First 4 bits of 0xAB = 1010 = 10
    assert_eq!(view.get(0, 4), 0x0A);
    // Next 4 bits = 1011 = 11
    assert_eq!(view.get(4, 4), 0x0B);
    // Full first byte
    assert_eq!(view.get(0, 8), 0xAB);
    // Bits across byte boundary: bits 4..12 = 1011_1100 = 0xBC
    assert_eq!(view.get(4, 8), 0xBC);
}

#[test]
fn test_bitview_16bit() {
    let data = [0x01, 0x02];
    let view = BitView::new(&data);

    // Big-endian: 0x0102
    assert_eq!(view.get_endian(0, 16, false), 0x0102);
    // Little-endian: bytes [0x01, 0x02] as LE = 0x0201
    assert_eq!(view.get_endian(0, 16, true), 0x0201);
}

#[test]
fn test_parse_mac_salt() {
    let salt = parse_mac_salt("CF:30:16:01:C7:2F");
    // Reversed: [0x2F, 0xC7, 0x01, 0x16, 0x30, 0xCF]
    assert_eq!(salt, [0x2F, 0xC7, 0x01, 0x16, 0x30, 0xCF]);
}

#[test]
fn test_gan_encrypt_decrypt_roundtrip() {
    let keys = rouxflow_bluetoothcube::protocol::gan_v2::ENCRYPTION_KEYS;
    let (key, iv) = derive_gan_keys(&keys, "AA:BB:CC:DD:EE:FF");

    let original = [0u8; 20];
    let encrypted = gan_encrypt(&key, &iv, &original);
    let decrypted = gan_decrypt(&key, &iv, &encrypted);
    assert_eq!(&decrypted, &original);
}
