use std::io::{Cursor, Read};

use image::RgbImage;
use openh264::decoder::{DecodeOptions, Decoder, DecoderConfig};
use openh264::formats::{RgbSliceU8, YUVSource};
use openh264::{nal_units, Error, OpenH264API};

#[test]
#[cfg(feature = "source")]
fn can_get_decoder() -> Result<(), Error> {
    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let _decoder = Decoder::with_api_config(api, config)?;

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn can_access_raw_api() -> Result<(), Error> {
    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config)?;

    unsafe {
        _ = decoder.raw_api();
    };

    Ok(())
}

#[test]
#[rustfmt::skip]
#[cfg(feature = "source")]
fn can_decode_single() -> Result<(), Error> {
    let sources = [
        include_bytes!("data/single_1920x1080_cabac.h264").as_slice(),
        include_bytes!("data/single_512x512_cabac.h264").as_slice(),
        include_bytes!("data/single_512x512_cavlc.h264").as_slice(),
    ];

    for src in sources.iter() {
        let api = OpenH264API::from_source();
        let config = DecoderConfig::default().debug(false);
        let mut decoder = Decoder::with_api_config(api, config)?;

        let yuv = decoder.decode(src)?.ok_or_else(|| Error::msg("Should not happen"))?;
        let mut rgb = vec![0; yuv.estimate_rgb_u8_size()];

        yuv.write_rgb8(&mut rgb);
    }

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn can_decode_multi_to_end() -> Result<(), Error> {
    let src = include_bytes!("data/multi_512x512.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default().debug(false);
    let mut decoder = Decoder::with_api_config(api, config)?;

    decoder.decode(src)?;

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn can_decode_multi_by_step() -> Result<(), Error> {
    let src = include_bytes!("data/multi_512x512.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config)?;

    let mut last_was_ok = false;

    for packet in nal_units(src) {
        last_was_ok = decoder.decode(packet).is_ok()
    }

    assert!(last_was_ok);

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn fails_on_truncated() -> Result<(), Error> {
    let src = include_bytes!("data/multi_512x512_truncated.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default().debug(false);
    let mut decoder = Decoder::with_api_config(api, config)?;

    assert!(decoder.decode(src).is_err());

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn what_goes_around_comes_around() -> Result<(), Error> {
    use openh264::encoder::{Encoder, EncoderConfig};
    use openh264::formats::YUVBuffer;

    let src = include_bytes!("data/lenna_128x128.rgb");
    let rgb_source = RgbSliceU8::new(src, (128, 128));
    let yuv = YUVBuffer::from_rgb_source(rgb_source);

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let mut encoder = Encoder::with_api_config(api, config)?;

    let stream = encoder.encode(&yuv)?;

    let api = OpenH264API::from_source();
    let src = stream.to_vec();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config)?;
    decoder.decode(&src)?;

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn decodes_file_requiring_flush_frame() -> Result<(), Error> {
    let src = include_bytes!("data/multi_1024x768.h264");
    let compare_data = include_bytes!("data/multi_1024x768.bmp");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config)?;
    let mut decoded = None;

    // Read packets in TODO: what? format.
    for packet in read_frame(src.as_slice()) {
        decoded = Some(decoder.decode(packet.as_slice())?);
    }

    // Generate image from decoded frame
    let decoded_frame = decoded.expect("No decoded data").expect("Image");
    let mut frame_data = vec![0u8; decoded_frame.estimate_rgb_u8_size()];
    decoded_frame.write_rgb8(frame_data.as_mut_slice());
    let decoded_frame = RgbImage::from_vec(1024, 768, frame_data).expect("Failed to convert into image buffer");

    // Get compare image
    let compare_data = Cursor::new(compare_data);
    let compare_data = image::load(compare_data, image::ImageFormat::Bmp)
        .expect("Image load failed")
        .into_rgb8();

    let result = image_compare::rgb_hybrid_compare(&decoded_frame, &compare_data).expect("Image dimensions differ");

    // Images should be 99% similar
    assert!(result.score > 0.99, "Image similarity score: {}", result.score);

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn decodes_file_with_bframes() -> Result<(), Error> {
    let src = include_bytes!("data/big_buck_bunny_640x360.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config)?;

    // Counting frames inside video
    let mut frames = 0;
    for packet in nal_units(src) {
        if decoder.decode_with_options(packet, DecodeOptions::NoFlush)?.is_some() {
            frames += 1;
        }
    }

    // Adding frames in buffer to the count
    frames += decoder.flush_all()?.len();

    // Video should have exactly 300 frames
    assert_eq!(frames, 300);

    Ok(())
}

// TODO: Can we remove this to use `to_bitstream_with_001_le` above?
// The packets in the file are written frame by frame
// the first 4 bytes are frame length in little endian
// followed by actual frame data
pub fn read_frame<T>(mut stream: T) -> impl Iterator<Item = Vec<u8>>
where
    T: Read,
{
    std::iter::from_fn(move || {
        let mut data = [0u8; 4];
        let result = stream.read_exact(data.as_mut_slice());
        if result.is_err() {
            return None;
        }

        let len = u32::from_le_bytes(data) as usize;
        let mut data = vec![0u8; len];

        let result = stream.read_exact(data.as_mut_slice());
        if result.is_err() {
            None
        } else {
            Some(data)
        }
    })
}
