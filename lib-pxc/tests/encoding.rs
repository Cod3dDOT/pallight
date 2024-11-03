mod common;

use common::{
    GRADIENT, RANDOM_RGB, REAL_IMAGE, REAL_IMAGE_HEIGHT, REAL_IMAGE_PALETTE_SIZE, REAL_IMAGE_WIDTH,
};
use lib_pxc::{decode, encode};

#[test]
fn test_encode_decode_rgb() {
    let encoded = encode(4, 4, &RANDOM_RGB).unwrap();
    assert!(!encoded.is_empty());

    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded.rgba_data, &RANDOM_RGB);

    assert_eq!(decoded.width, 4);
    assert_eq!(decoded.height, 4);
    assert_eq!(decoded.palette.len(), 3);
}

#[test]
fn test_encode_decode_real_image() {
    let encoded = encode(REAL_IMAGE_WIDTH, REAL_IMAGE_HEIGHT, &REAL_IMAGE).unwrap();
    assert!(!encoded.is_empty());

    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded.rgba_data, &REAL_IMAGE);

    assert_eq!(decoded.width, REAL_IMAGE_WIDTH);
    assert_eq!(decoded.height, REAL_IMAGE_HEIGHT);
    assert_eq!(decoded.palette.len(), REAL_IMAGE_PALETTE_SIZE);
}

#[test]
fn test_encode_decode_repeating_color() {
    const WIDTH: u16 = 4;
    const HEIGHT: u16 = 4;

    let data = vec![255, 0, 0, 255].repeat((WIDTH * HEIGHT).try_into().unwrap()); // 4x4 red image

    let encoded = encode(4, 4, &data).unwrap();

    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded.rgba_data, data);

    assert_eq!(decoded.width, WIDTH);
    assert_eq!(decoded.height, HEIGHT);
    assert_eq!(decoded.palette.len(), 1);
    assert_eq!(decoded.palette[0], [255, 0, 0, 255]);
}

#[test]
fn test_encode_decode_gradients() {
    const WIDTH: u16 = 16;
    const HEIGHT: u16 = 16;

    let encoded = encode(WIDTH, HEIGHT, &GRADIENT);

    if let Err(ref e) = encoded {
        println!("Encode error: {:?}", e);
    }
    assert!(encoded.is_ok());

    let encoded = encoded.unwrap();

    assert!(!encoded.is_empty());

    let decoded = decode(&encoded);

    if let Err(ref e) = decoded {
        println!("Decode error: {:?}", e);
    }
    assert!(decoded.is_ok(), "Decoding failed");

    let decoded = decoded.unwrap();
    assert_eq!(decoded.rgba_data, &GRADIENT);

    assert_eq!(decoded.width, WIDTH);
    assert_eq!(decoded.height, HEIGHT);
    assert_eq!(decoded.palette.len(), 256);
}
