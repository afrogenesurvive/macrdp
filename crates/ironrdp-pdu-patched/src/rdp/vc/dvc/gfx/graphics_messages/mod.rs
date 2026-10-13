mod client;
mod server;

mod avc_messages;
use bitflags::bitflags;

#[rustfmt::skip] // do not re-order this
pub use avc_messages::{Avc420BitmapStream, Avc444BitmapStream, Encoding, QuantQuality};
pub use client::{CacheImportReplyPdu, CapabilitiesAdvertisePdu, FrameAcknowledgePdu, QueueDepth};
use ironrdp_core::{
    cast_length, ensure_fixed_part_size, ensure_size, invalid_field_err, Decode, DecodeResult, Encode, EncodeResult,
    ReadCursor, WriteCursor,
};
pub use server::{
    CacheToSurfacePdu, CapabilitiesConfirmPdu, Codec1Type, Codec2Type, CreateSurfacePdu, DeleteEncodingContextPdu,
    DeleteSurfacePdu, EndFramePdu, EvictCacheEntryPdu, MapSurfaceToOutputPdu, MapSurfaceToScaledOutputPdu,
    MapSurfaceToScaledWindowPdu, PixelFormat, ResetGraphicsPdu, SolidFillPdu, StartFramePdu, SurfaceToCachePdu,
    SurfaceToSurfacePdu, Timestamp, WireToSurface1Pdu, WireToSurface2Pdu,
};

use super::RDP_GFX_HEADER_SIZE;

const CAPABILITY_SET_HEADER_SIZE: usize = 8;

const V10_1_RESERVED: u128 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilitySet {
    V8 { flags: CapabilitiesV8Flags },
    V8_1 { flags: CapabilitiesV81Flags },
    V10 { flags: CapabilitiesV10Flags },
    V10_1,
    V10_2 { flags: CapabilitiesV10Flags },
    V10_3 { flags: CapabilitiesV103Flags },
    V10_4 { flags: CapabilitiesV104Flags },
    V10_5 { flags: CapabilitiesV104Flags },
    V10_6 { flags: CapabilitiesV104Flags },
    V10_6Err { flags: CapabilitiesV104Flags },
    V10_7 { flags: CapabilitiesV107Flags },
}

impl CapabilitySet {
    const NAME: &'static str = "GfxCapabilitySet";

    /// Wire `version` field corresponding to this typed variant.
    pub fn version(&self) -> CapabilityVersion {
        match self {
            CapabilitySet::V8 { .. } => CapabilityVersion::V8,
            CapabilitySet::V8_1 { .. } => CapabilityVersion::V8_1,
            CapabilitySet::V10 { .. } => CapabilityVersion::V10,
            CapabilitySet::V10_1 => CapabilityVersion::V10_1,
            CapabilitySet::V10_2 { .. } => CapabilityVersion::V10_2,
            CapabilitySet::V10_3 { .. } => CapabilityVersion::V10_3,
            CapabilitySet::V10_4 { .. } => CapabilityVersion::V10_4,
            CapabilitySet::V10_5 { .. } => CapabilityVersion::V10_5,
            CapabilitySet::V10_6 { .. } => CapabilityVersion::V10_6,
            CapabilitySet::V10_6Err { .. } => CapabilityVersion::V10_6Err,
            CapabilitySet::V10_7 { .. } => CapabilityVersion::V10_7,
        }
    }
}

impl Encode for CapabilitySet {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> EncodeResult<()> {
        ensure_size!(in: dst, size: self.size());

        dst.write_u32(self.version().0);
        dst.write_u32(cast_length!("dataLength", self.size() - CAPABILITY_SET_HEADER_SIZE)?);

        match self {
            CapabilitySet::V8 { flags } => dst.write_u32(flags.bits()),
            CapabilitySet::V8_1 { flags } => dst.write_u32(flags.bits()),
            CapabilitySet::V10 { flags } => dst.write_u32(flags.bits()),
            CapabilitySet::V10_1 => dst.write_u128(V10_1_RESERVED),
            CapabilitySet::V10_2 { flags } => dst.write_u32(flags.bits()),
            CapabilitySet::V10_3 { flags } => dst.write_u32(flags.bits()),
            CapabilitySet::V10_4 { flags } => dst.write_u32(flags.bits()),
            CapabilitySet::V10_5 { flags } => dst.write_u32(flags.bits()),
            CapabilitySet::V10_6 { flags } => dst.write_u32(flags.bits()),
            CapabilitySet::V10_6Err { flags } => dst.write_u32(flags.bits()),
            CapabilitySet::V10_7 { flags } => dst.write_u32(flags.bits()),
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn size(&self) -> usize {
        CAPABILITY_SET_HEADER_SIZE
            + match self {
                CapabilitySet::V8 { .. }
                | CapabilitySet::V8_1 { .. }
                | CapabilitySet::V10 { .. }
                | CapabilitySet::V10_2 { .. }
                | CapabilitySet::V10_3 { .. }
                | CapabilitySet::V10_4 { .. }
                | CapabilitySet::V10_5 { .. }
                | CapabilitySet::V10_6 { .. }
                | CapabilitySet::V10_6Err { .. }
                | CapabilitySet::V10_7 { .. } => 4,
                CapabilitySet::V10_1 => 16,
            }
    }
}

/// Lossless wire representation of the `RDPGFX_CAPSET` structure
/// ([MS-RDPEGFX] 2.2.1.6).
///
/// The raw `version` is kept next to the raw body `data`, so a capability set
/// whose version this build does not recognize is preserved and re-encoded
/// byte-for-byte. This matters because `CapabilitiesAdvertise` carries a *list*
/// of capability sets: failing to decode one of them used to abort the whole
/// PDU, which aborts EGFX negotiation and silently disables video.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawCapabilitySet {
    pub version: CapabilityVersion,
    pub data: Vec<u8>,
}

impl RawCapabilitySet {
    const NAME: &'static str = "GfxCapabilitySet";

    const FIXED_PART_SIZE: usize = CAPABILITY_SET_HEADER_SIZE;

    pub fn new(version: CapabilityVersion, data: Vec<u8>) -> Self {
        Self { version, data }
    }

    /// Parse the body into a typed [`CapabilitySet`] when the version is known
    /// to this build. Returns `Ok(None)` for versions this build does not know,
    /// leaving the raw bytes available via [`Self::data`].
    pub fn parsed(&self) -> DecodeResult<Option<CapabilitySet>> {
        let mut cur = ReadCursor::new(&self.data);

        let size = match self.version {
            CapabilityVersion::V8
            | CapabilityVersion::V8_1
            | CapabilityVersion::V10
            | CapabilityVersion::V10_2
            | CapabilityVersion::V10_3
            | CapabilityVersion::V10_4
            | CapabilityVersion::V10_5
            | CapabilityVersion::V10_6
            | CapabilityVersion::V10_6Err
            | CapabilityVersion::V10_7 => 4,
            CapabilityVersion::V10_1 => 16,
            _ => return Ok(None),
        };

        ensure_size!(in: cur, size: size);

        let cap = match self.version {
            CapabilityVersion::V8 => CapabilitySet::V8 {
                flags: CapabilitiesV8Flags::from_bits_truncate(cur.read_u32()),
            },
            CapabilityVersion::V8_1 => CapabilitySet::V8_1 {
                flags: CapabilitiesV81Flags::from_bits_truncate(cur.read_u32()),
            },
            CapabilityVersion::V10 => CapabilitySet::V10 {
                flags: CapabilitiesV10Flags::from_bits_truncate(cur.read_u32()),
            },
            CapabilityVersion::V10_1 => {
                cur.read_u128();

                CapabilitySet::V10_1
            }
            CapabilityVersion::V10_2 => CapabilitySet::V10_2 {
                flags: CapabilitiesV10Flags::from_bits_truncate(cur.read_u32()),
            },
            CapabilityVersion::V10_3 => CapabilitySet::V10_3 {
                flags: CapabilitiesV103Flags::from_bits_truncate(cur.read_u32()),
            },
            CapabilityVersion::V10_4 => CapabilitySet::V10_4 {
                flags: CapabilitiesV104Flags::from_bits_truncate(cur.read_u32()),
            },
            CapabilityVersion::V10_5 => CapabilitySet::V10_5 {
                flags: CapabilitiesV104Flags::from_bits_truncate(cur.read_u32()),
            },
            CapabilityVersion::V10_6 => CapabilitySet::V10_6 {
                flags: CapabilitiesV104Flags::from_bits_truncate(cur.read_u32()),
            },
            CapabilityVersion::V10_6Err => CapabilitySet::V10_6Err {
                flags: CapabilitiesV104Flags::from_bits_truncate(cur.read_u32()),
            },
            CapabilityVersion::V10_7 => CapabilitySet::V10_7 {
                flags: CapabilitiesV107Flags::from_bits_truncate(cur.read_u32()),
            },
            _ => return Ok(None),
        };

        Ok(Some(cap))
    }
}

impl Encode for RawCapabilitySet {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> EncodeResult<()> {
        ensure_size!(in: dst, size: self.size());

        dst.write_u32(self.version.0);
        dst.write_u32(cast_length!("dataLength", self.data.len())?);
        dst.write_slice(&self.data);

        Ok(())
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn size(&self) -> usize {
        Self::FIXED_PART_SIZE + self.data.len()
    }
}

impl<'de> Decode<'de> for RawCapabilitySet {
    fn decode(src: &mut ReadCursor<'de>) -> DecodeResult<Self> {
        ensure_fixed_part_size!(in: src);

        let version = CapabilityVersion(src.read_u32());
        let data_length: usize = cast_length!("dataLength", src.read_u32())?;

        ensure_size!(in: src, size: data_length);
        let data = src.read_slice(data_length).to_vec();

        // Deliberately do *not* validate the version here: an unrecognized
        // version must not fail the whole CapabilitiesAdvertise PDU.
        Ok(Self { version, data })
    }
}

impl From<&CapabilitySet> for RawCapabilitySet {
    fn from(cap: &CapabilitySet) -> Self {
        let mut buf = vec![0u8; cap.size()];
        let mut cur = WriteCursor::new(&mut buf);
        cap.encode(&mut cur)
            .expect("buffer is exactly cap.size() bytes; the write cannot fail");

        let data = buf[CAPABILITY_SET_HEADER_SIZE..].to_vec();

        Self {
            version: cap.version(),
            data,
        }
    }
}

impl<'de> Decode<'de> for CapabilitySet {
    fn decode(src: &mut ReadCursor<'de>) -> DecodeResult<Self> {
        let raw = RawCapabilitySet::decode(src)?;

        raw.parsed()?
            .ok_or_else(|| invalid_field_err!("version", "unhandled version"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub xa: u8,
}

impl Color {
    const NAME: &'static str = "GfxColor";

    const FIXED_PART_SIZE: usize = 4 /* BGRA */;
}

impl Encode for Color {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> EncodeResult<()> {
        ensure_fixed_part_size!(in: dst);

        dst.write_u8(self.b);
        dst.write_u8(self.g);
        dst.write_u8(self.r);
        dst.write_u8(self.xa);

        Ok(())
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn size(&self) -> usize {
        Self::FIXED_PART_SIZE
    }
}

impl<'de> Decode<'de> for Color {
    fn decode(src: &mut ReadCursor<'de>) -> DecodeResult<Self> {
        ensure_fixed_part_size!(in: src);

        let b = src.read_u8();
        let g = src.read_u8();
        let r = src.read_u8();
        let xa = src.read_u8();

        Ok(Self { b, g, r, xa })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    const NAME: &'static str = "GfxPoint";

    const FIXED_PART_SIZE: usize = 2 /* X */ + 2 /* Y */;
}

impl Encode for Point {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> EncodeResult<()> {
        ensure_fixed_part_size!(in: dst);

        dst.write_u16(self.x);
        dst.write_u16(self.y);

        Ok(())
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn size(&self) -> usize {
        Self::FIXED_PART_SIZE
    }
}

impl<'de> Decode<'de> for Point {
    fn decode(src: &mut ReadCursor<'de>) -> DecodeResult<Self> {
        ensure_fixed_part_size!(in: src);

        let x = src.read_u16();
        let y = src.read_u16();

        Ok(Self { x, y })
    }
}

/// Capability set version, as advertised in the `RDPGFX_CAPSET` structure
/// ([MS-RDPEGFX] 2.2.1.6).
///
/// This is a plain `u32` wrapper instead of a closed enum so that versions a
/// build does not recognize can still be decoded and re-encoded losslessly
/// (see [`RawCapabilitySet`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityVersion(pub u32);

#[allow(non_upper_case_globals)]
impl CapabilityVersion {
    pub const V8: Self = Self(0x8_0004);
    pub const V8_1: Self = Self(0x8_0105);
    pub const V10: Self = Self(0xa_0002);
    pub const V10_1: Self = Self(0xa_0100);
    pub const V10_2: Self = Self(0xa_0200);
    pub const V10_3: Self = Self(0xa_0301);
    pub const V10_4: Self = Self(0xa_0400);
    pub const V10_5: Self = Self(0xa_0502);
    pub const V10_6: Self = Self(0xa_0600); // [MS-RDPEGFX-errata]
    pub const V10_6Err: Self = Self(0xa_0601); // defined similar to FreeRDP to maintain best compatibility
    pub const V10_7: Self = Self(0xa_0701);

    /// The matching version, when this build knows how to interpret it.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x8_0004 => Some(Self::V8),
            0x8_0105 => Some(Self::V8_1),
            0xa_0002 => Some(Self::V10),
            0xa_0100 => Some(Self::V10_1),
            0xa_0200 => Some(Self::V10_2),
            0xa_0301 => Some(Self::V10_3),
            0xa_0400 => Some(Self::V10_4),
            0xa_0502 => Some(Self::V10_5),
            0xa_0600 => Some(Self::V10_6),
            0xa_0601 => Some(Self::V10_6Err),
            0xa_0701 => Some(Self::V10_7),
            _ => None,
        }
    }

    /// Whether this build knows how to interpret the version's body.
    pub fn is_known(self) -> bool {
        Self::from_u32(self.0).is_some()
    }
}

impl From<CapabilityVersion> for u32 {
    fn from(version: CapabilityVersion) -> Self {
        version.0
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CapabilitiesV8Flags: u32  {
        const THIN_CLIENT = 0x1;
        const SMALL_CACHE = 0x2;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CapabilitiesV81Flags: u32  {
        const THIN_CLIENT = 0x01;
        const SMALL_CACHE = 0x02;
        const AVC420_ENABLED = 0x10;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CapabilitiesV10Flags: u32 {
        const SMALL_CACHE = 0x02;
        const AVC_DISABLED = 0x20;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CapabilitiesV103Flags: u32  {
        const AVC_DISABLED = 0x20;
        const AVC_THIN_CLIENT = 0x40;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CapabilitiesV104Flags: u32  {
        const SMALL_CACHE = 0x02;
        const AVC_DISABLED = 0x20;
        const AVC_THIN_CLIENT = 0x40;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CapabilitiesV107Flags: u32  {
        const SMALL_CACHE = 0x02;
        const AVC_DISABLED = 0x20;
        const AVC_THIN_CLIENT = 0x40;
        const SCALEDMAP_DISABLE = 0x80;
    }
}

#[cfg(test)]
mod tests {
    use ironrdp_core::{decode, encode_vec};

    use super::*;

    /// Real `RDPGFX_CAPS_ADVERTISE` body captured from a Microsoft "Windows App"
    /// mobile client (device `Z2450`): `capsSetCount` (u16) followed by eight
    /// capability sets, the last two using versions (`0x000B0101`, `0x000B0300`)
    /// newer than any this crate knows.
    const ADVERTISE_BODY: &[u8] = &[
        8, 0, // capsSetCount
        4, 0, 8, 0, 4, 0, 0, 0, 2, 0, 0, 0, // V8 (0x00080004), flags 0x02
        5, 1, 8, 0, 4, 0, 0, 0, 2, 0, 0, 0, // V8.1 (0x00080105), flags 0x02
        2, 0, 10, 0, 4, 0, 0, 0, 34, 0, 0, 0, // V10 (0x000A0002), flags 0x22
        0, 2, 10, 0, 4, 0, 0, 0, 34, 0, 0, 0, // V10.2 (0x000A0200), flags 0x22
        1, 3, 10, 0, 4, 0, 0, 0, 32, 0, 0, 0, // V10.3 (0x000A0301), flags 0x20
        0, 4, 10, 0, 4, 0, 0, 0, 34, 0, 0, 0, // V10.4 (0x000A0400), flags 0x22
        1, 1, 11, 0, 4, 0, 0, 0, 162, 1, 0, 0, // 0x000B0101, flags 0x1A2
        0, 3, 11, 0, 4, 0, 0, 0, 162, 1, 0, 0, // 0x000B0300, flags 0x1A2
    ];

    #[test]
    fn decodes_advertise_with_unknown_versions() {
        let pdu = decode::<CapabilitiesAdvertisePdu>(ADVERTISE_BODY).expect("advertise must decode");

        assert_eq!(pdu.0.len(), 8, "all eight capability sets must be preserved");

        let versions: Vec<u32> = pdu.0.iter().map(|set| set.version.0).collect();
        assert_eq!(
            versions,
            vec![
                0x0008_0004,
                0x0008_0105,
                0x000A_0002,
                0x000A_0200,
                0x000A_0301,
                0x000A_0400,
                0x000B_0101,
                0x000B_0300,
            ]
        );

        // Recognized versions parse into typed variants …
        assert_eq!(
            pdu.0[0].parsed().unwrap(),
            Some(CapabilitySet::V8 {
                flags: CapabilitiesV8Flags::SMALL_CACHE,
            })
        );
        assert_eq!(
            pdu.0[5].parsed().unwrap(),
            Some(CapabilitySet::V10_4 {
                flags: CapabilitiesV104Flags::SMALL_CACHE | CapabilitiesV104Flags::AVC_DISABLED,
            })
        );

        // … while the two newer versions are preserved but not interpretable.
        assert_eq!(pdu.0[6].parsed().unwrap(), None);
        assert_eq!(pdu.0[7].parsed().unwrap(), None);
        assert_eq!(pdu.0[7].data, [0xA2, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn re_encodes_advertise_byte_for_byte() {
        let pdu = decode::<CapabilitiesAdvertisePdu>(ADVERTISE_BODY).unwrap();

        assert_eq!(encode_vec(&pdu).unwrap(), ADVERTISE_BODY);
    }

    #[test]
    fn unknown_version_round_trips_through_confirm() {
        let raw = RawCapabilitySet::new(CapabilityVersion(0x000B_0300), vec![0xA2, 0x01, 0x00, 0x00]);
        let pdu = CapabilitiesConfirmPdu(raw);

        let encoded = encode_vec(&pdu).unwrap();
        assert_eq!(
            encoded,
            [0x00, 0x03, 0x0B, 0x00, 0x04, 0x00, 0x00, 0x00, 0xA2, 0x01, 0x00, 0x00]
        );

        assert_eq!(decode::<CapabilitiesConfirmPdu>(&encoded).unwrap(), pdu);
    }

    #[test]
    fn capability_set_decode_still_rejects_unknown_versions() {
        // `CapabilitySet::decode` keeps its strict behavior: it is the typed view,
        // not the lossless wire representation.
        let bytes = [0x01, 0x01, 0x0B, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        assert!(decode::<CapabilitySet>(&bytes).is_err());
    }
}
