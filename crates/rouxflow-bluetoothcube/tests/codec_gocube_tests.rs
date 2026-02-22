use rouxflow_bluetoothcube::codec::gocube::GoCubeCodec;
use rouxflow_bluetoothcube::codec::{CubeEvent, CubeProtocol};
use rouxflow_core::cube::Face;

#[test]
fn test_frame_validation() {
    let mut codec = GoCubeCodec::new();

    // Valid frame with battery type
    let frame = vec![0x2A, 0x01, 5, 85, 0x0D, 0x0A];
    let events = codec.decode_event(&frame);
    assert_eq!(events.len(), 1);
    match &events[0] {
        CubeEvent::Battery { level } => assert_eq!(*level, 85),
        _ => panic!("Expected Battery event"),
    }

    // Invalid frame (wrong start)
    let bad = vec![0x00, 0x01, 5, 85, 0x0D, 0x0A];
    assert!(codec.decode_event(&bad).is_empty());
}

#[test]
fn test_move_decode() {
    let mut codec = GoCubeCodec::new();

    // Move byte: face=2 (U in GoCube), dir=0 (CW)
    // GoCube face 2 → AXIS_PERM[2] = 0 → U in standard
    let frame = vec![0x2A, 0x01, 1, 4, 0, 0x0D, 0x0A]; // move_byte=4 = face 2, dir 0
    let events = codec.decode_event(&frame);
    assert_eq!(events.len(), 1);
    match &events[0] {
        CubeEvent::Move { face, direction, .. } => {
            assert_eq!(*face, Face::U);
            assert_eq!(*direction, 1); // CW
        }
        _ => panic!("Expected Move event"),
    }
}
