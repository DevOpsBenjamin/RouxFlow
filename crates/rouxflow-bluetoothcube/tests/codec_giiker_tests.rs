use rouxflow_bluetoothcube::codec::giiker::GiikerCodec;
use rouxflow_bluetoothcube::protocol::giiker::ENCRYPTION_MARKER;

#[test]
fn test_decrypt_unencrypted() {
    // Unencrypted packet (byte 18 != 0xA7)
    let mut data = vec![0u8; 20];
    data[0] = 0x12;
    data[18] = 0x00;
    let result = GiikerCodec::decrypt_packet(&data);
    assert_eq!(result.len(), 20);
    assert_eq!(result[0], 0x12);
}

#[test]
fn test_decrypt_encrypted() {
    use rouxflow_bluetoothcube::protocol::giiker::KEY_TABLE;
    // Encrypted packet (byte 18 == 0xA7)
    let mut data = vec![0u8; 20];
    data[18] = ENCRYPTION_MARKER;
    data[19] = 0x00; // k1=0, k2=0
    let result = GiikerCodec::decrypt_packet(&data);
    assert_eq!(result.len(), 18);
    // Each byte should have KEY_TABLE[i] + KEY_TABLE[i] added (k1=k2=0)
    for i in 0..18 {
        assert_eq!(result[i], KEY_TABLE[i].wrapping_add(KEY_TABLE[i]));
    }
}

#[test]
fn test_to_hex_nibbles() {
    let nibbles = GiikerCodec::to_hex_nibbles(&[0xAB, 0xCD]);
    assert_eq!(nibbles, vec![0xA, 0xB, 0xC, 0xD]);
}
