use rouxflow_bluetoothcube::codec::moyu_v3::MoYuV3Codec;
use rouxflow_core::cube::Face;

#[test]
fn test_decode_move() {
    assert_eq!(MoYuV3Codec::decode_move(0), Some((Face::F, 1))); // F CW
    assert_eq!(MoYuV3Codec::decode_move(1), Some((Face::F, -1))); // F CCW
    assert_eq!(MoYuV3Codec::decode_move(2), Some((Face::B, 1))); // B CW
    assert_eq!(MoYuV3Codec::decode_move(4), Some((Face::U, 1))); // U CW
    assert_eq!(MoYuV3Codec::decode_move(11), Some((Face::R, -1))); // R CCW
    assert_eq!(MoYuV3Codec::decode_move(12), None); // invalid
}
