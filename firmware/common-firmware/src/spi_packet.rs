use crc::{CRC_32_ISO_HDLC, Crc};

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

pub const PACKET_SIZE: usize = 256;
pub const HEADER_SIZE: usize = 4;
pub const CRC_SIZE: usize = 4;
pub const DATA_SIZE: usize = PACKET_SIZE - HEADER_SIZE - CRC_SIZE;

#[derive(Debug, Clone, Copy)]
pub struct SpiPacket {
    pub packet_number: u8,
    pub total_packets: u8,
    pub data_length: u8,
    pub data: [u8; DATA_SIZE],
    pub crc: u32,
}

#[derive(Debug)]
pub enum SpiPacketError {
    InvalidSize,
    InvalidLength,
    InvalidPacketNumber,
    InvalidTotalPackets,
    TooManyPackets,
    CrcMismatch,
}

impl SpiPacket {
    pub fn new(packet_number: u8, total_packets: u8, data: &[u8]) -> Result<Self, SpiPacketError> {
        if data.len() > DATA_SIZE || data.len() > u8::MAX as usize {
            return Err(SpiPacketError::InvalidLength);
        }

        let mut packet = Self {
            packet_number,
            total_packets,
            data_length: data.len() as u8,
            data: [0u8; DATA_SIZE],
            crc: 0,
        };

        packet.data[..data.len()].copy_from_slice(data);
        packet.crc = packet.calculate_crc();

        Ok(packet)
    }

    fn calculate_crc(&self) -> u32 {
        let mut digest = CRC32.digest();

        digest.update(&[self.packet_number]);
        digest.update(&[self.total_packets]);
        digest.update(&[self.data_length]);
        digest.update(&self.data[..self.data_length as usize]);

        digest.finalize()
    }

    pub fn to_bytes(&self) -> [u8; PACKET_SIZE] {
        let mut output = [0u8; PACKET_SIZE];

        output[0] = self.packet_number;
        output[1] = self.total_packets;
        output[2] = self.data_length;

        // output[3] is reserved and remains 0.

        output[HEADER_SIZE..HEADER_SIZE + DATA_SIZE].copy_from_slice(&self.data);

        let crc_offset = HEADER_SIZE + DATA_SIZE;

        output[crc_offset..PACKET_SIZE].copy_from_slice(&self.crc.to_le_bytes());

        output
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SpiPacketError> {
        if bytes.len() != PACKET_SIZE {
            return Err(SpiPacketError::InvalidSize);
        }

        let packet_number = bytes[0];
        let total_packets = bytes[1];
        let data_length = bytes[2];

        println!("from bytes");
        println!("{:?}", packet_number);
        println!("{:?}", total_packets);
        println!("{:?}", data_length);

        if total_packets == 0 {
            return Err(SpiPacketError::InvalidTotalPackets);
        }

        if packet_number >= total_packets {
            return Err(SpiPacketError::InvalidPacketNumber);
        }

        if data_length as usize > DATA_SIZE {
            return Err(SpiPacketError::InvalidLength);
        }

        let mut data = [0u8; DATA_SIZE];

        data.copy_from_slice(&bytes[HEADER_SIZE..HEADER_SIZE + DATA_SIZE]);

        let crc_offset = HEADER_SIZE + DATA_SIZE;

        let crc = u32::from_le_bytes([
            bytes[crc_offset],
            bytes[crc_offset + 1],
            bytes[crc_offset + 2],
            bytes[crc_offset + 3],
        ]);

        let packet = Self {
            packet_number,
            total_packets,
            data_length,
            data,
            crc,
        };

        if packet.calculate_crc() != crc {
            return Err(SpiPacketError::CrcMismatch);
        }

        Ok(packet)
    }

    pub fn payload(&self) -> &[u8] {
        &self.data[..self.data_length as usize]
    }

    pub fn is_last(&self) -> bool {
        self.packet_number == self.total_packets - 1
    }
}

#[derive(Debug)]
pub struct SpiPackets {
    packets: Vec<SpiPacket>,
}

impl SpiPackets {
    pub fn from_vec(packets: Vec<SpiPacket>) -> Self {
        Self {
            packets,
        }
    }

    pub fn push(&mut self, packet: SpiPacket) {
        self.packets.push(packet);
    }

    /// Create an empty packet collection that will be filled incrementally.
    pub fn from_payload(data: &[u8]) -> Result<Self, SpiPacketError> {
        let total_packets = data.len().div_ceil(DATA_SIZE);

        // An empty payload is represented by one empty packet.
        let total_packets = total_packets.max(1);

        if total_packets > u8::MAX as usize {
            return Err(SpiPacketError::TooManyPackets);
        }

        let total_packets = total_packets as u8;

        let packets = data
            .chunks(DATA_SIZE)
            .enumerate()
            .map(|(index, chunk)| SpiPacket::new(index as u8, total_packets, chunk))
            .collect::<Result<Vec<_>, _>>()?;

        let packets = if packets.is_empty() {
            vec![SpiPacket::new(0, 1, &[])?]
        } else {
            packets
        };

        Ok(Self { packets })
    }
    pub fn len(&self) -> usize {
        self.packets.len()
    }

    pub fn packets(&self) -> &[SpiPacket] {
        &self.packets
    }

    pub fn iter(&self) -> impl Iterator<Item = &SpiPacket> {
        self.packets.iter()
    }

    pub fn payload(&self) -> Vec<u8> {
        let total_size: usize = self.packets.iter().map(|p| p.data_length as usize).sum();

        let mut output = Vec::with_capacity(total_size);

        for packet in &self.packets {
            output.extend_from_slice(packet.payload());
        }

        output
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(self.packets.len() * PACKET_SIZE);

        for packet in &self.packets {
            output.extend_from_slice(&packet.to_bytes());
        }

        output
    }
}
