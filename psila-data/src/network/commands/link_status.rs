use crate::error::Error;
use crate::pack::{Pack, PackFixed};
use crate::NetworkAddress;

const INCOMING_COST_MASK: u8 = 0b0000_0111;
const OUTGOING_COST_MASK: u8 = 0b0111_0000;
const LINK_STATUS_ENTRY_SIZE: usize = 3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LinkStatusEntry {
    pub address: NetworkAddress,
    pub incoming_cost: u8,
    pub outgoing_cost: u8,
}

impl PackFixed<LinkStatusEntry, Error> for LinkStatusEntry {
    fn pack(&self, data: &mut [u8]) -> Result<(), Error> {
        if data.len() != LINK_STATUS_ENTRY_SIZE {
            return Err(Error::WrongNumberOfBytes);
        }
        assert!(self.incoming_cost < 16);
        assert!(self.outgoing_cost < 16);
        self.address.pack(&mut data[0..2])?;
        data[2] = self.incoming_cost | self.outgoing_cost << 4;
        Ok(())
    }

    fn unpack(data: &[u8]) -> Result<Self, Error> {
        if data.len() != LINK_STATUS_ENTRY_SIZE {
            return Err(Error::WrongNumberOfBytes);
        }
        let address = NetworkAddress::unpack(&data[0..2])?;
        let incoming_cost = data[2] & INCOMING_COST_MASK;
        let outgoing_cost = (data[2] & OUTGOING_COST_MASK) >> 4;
        Ok(LinkStatusEntry {
            address,
            incoming_cost,
            outgoing_cost,
        })
    }
}

const NUMBER_OF_ENTRIES_MASK: u8 = 0b0001_1111;
const FIRST_FRAME: u8 = 0b0010_0000;
const LAST_FRAME: u8 = 0b0100_0000;

#[derive(Clone, Debug, PartialEq)]
pub struct LinkStatus {
    pub first_frame: bool,
    pub last_frame: bool,
    pub entries: Vec<LinkStatusEntry>,
}

impl Pack<LinkStatus, Error> for LinkStatus {
    fn pack(&self, data: &mut [u8]) -> Result<usize, Error> {
        assert!(self.entries.len() < 32);
        if data.len() < (1 + (self.entries.len() * LINK_STATUS_ENTRY_SIZE)) {
            return Err(Error::WrongNumberOfBytes);
        }
        let mut offset = 1;
        let num_entries = self.entries.len() as u8;
        data[0] = num_entries
            | if self.first_frame { FIRST_FRAME } else { 0 }
            | if self.last_frame { LAST_FRAME } else { 0 };
        for entry in self.entries.iter() {
            entry.pack(&mut data[offset..offset + LINK_STATUS_ENTRY_SIZE])?;
            offset += LINK_STATUS_ENTRY_SIZE;
        }
        Ok(offset)
    }

    fn unpack(data: &[u8]) -> Result<(Self, usize), Error> {
        if data.is_empty() {
            return Err(Error::WrongNumberOfBytes);
        }
        let num_entries = (data[0] & NUMBER_OF_ENTRIES_MASK) as usize;
        if data.len() < (1 + (num_entries * LINK_STATUS_ENTRY_SIZE)) {
            return Err(Error::WrongNumberOfBytes);
        }
        let first_frame = (data[0] & FIRST_FRAME) == FIRST_FRAME;
        let last_frame = (data[0] & LAST_FRAME) == LAST_FRAME;
        let mut offset = 1;
        let mut entries: Vec<LinkStatusEntry> = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            let entry = LinkStatusEntry::unpack(&data[offset..offset + LINK_STATUS_ENTRY_SIZE])?;
            entries.push(entry);
            offset += LINK_STATUS_ENTRY_SIZE;
        }

        Ok((
            LinkStatus {
                first_frame,
                last_frame,
                entries,
            },
            offset,
        ))
    }
}
