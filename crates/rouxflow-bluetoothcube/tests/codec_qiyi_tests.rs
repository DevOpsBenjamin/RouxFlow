use rouxflow_bluetoothcube::codec::qiyi::QiYiCodec;
use rouxflow_core::cube::Face;

#[test]
fn test_crc16_modbus() {
    use rouxflow_bluetoothcube::codec::qiyi::crc16_modbus;

    // Known test vector
    let data = [0xFE, 0x05, 0x00];
    let crc = crc16_modbus(&data);
    assert_ne!(crc, 0); // Just verify it computes something

    // CRC of a full frame should be 0
    let mut frame = vec![0xFE, 0x05, 0x00];
    let crc = crc16_modbus(&frame);
    frame.push((crc & 0xFF) as u8);
    frame.push((crc >> 8) as u8);
    assert_eq!(crc16_modbus(&frame), 0);
}

#[test]
fn test_parse_facelets() {
    // All zeros = all 'L' (color 0)
    let data = [0u8; 27];
    let result = QiYiCodec::parse_facelets(&data);
    assert_eq!(result.len(), 54);
    assert!(result.chars().all(|c| c == 'L'));
}

#[test]
fn test_decode_move() {
    // move_byte 1: idx=0, axis_map[0]=4=L, odd=CW
    assert_eq!(QiYiCodec::decode_move(1), Some((Face::L, 1)));
    // move_byte 2: idx=0, axis_map[0]=4=L, even=CCW
    assert_eq!(QiYiCodec::decode_move(2), Some((Face::L, -1)));
    // move_byte 3: idx=1, axis_map[1]=1=R, odd=CW
    assert_eq!(QiYiCodec::decode_move(3), Some((Face::R, 1)));
    // move_byte 7: idx=3, axis_map[3]=0=U, odd=CW
    assert_eq!(QiYiCodec::decode_move(7), Some((Face::U, 1)));
    // move_byte 0: invalid
    assert_eq!(QiYiCodec::decode_move(0), None);
    // move_byte 13: invalid
    assert_eq!(QiYiCodec::decode_move(13), None);
}

#[test]
fn test_ecb_roundtrip() {
    let codec = QiYiCodec::new("AA:BB:CC:DD:EE:FF");
    let data = vec![0u8; 16]; // One block
    let encrypted = codec.ecb_encrypt(&data);
    let decrypted = codec.ecb_decrypt(&encrypted);
    assert_eq!(decrypted, data);
}
