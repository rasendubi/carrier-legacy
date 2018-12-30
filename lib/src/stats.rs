use std::collections::HashMap;
use identity::Identity;
use proto;

#[derive(Default)]
pub struct PacketCounter {
    /// sent packets (for proxy this is packets sent from initiator->responder)
    pub tx: u64,
    /// received packets (for proxy this packets sent from responder->initiator)
    pub rx: u64,

    pub responder: Option<Identity>,
    pub initiator: Option<Identity>,
}

#[derive(Default)]
pub struct Stats {
    v: HashMap<Identity, (u64, u64, u64, u64)>,
}

impl Stats {
    pub fn traffic(&mut self, i: Identity, tx: u64, rx: u64, bx: u64, dc: bool) {
        let (ref mut rx_, ref mut tx_, ref mut bx_, ref mut dcs_) = self.v.entry(i).or_insert((0,0,0,0));
        *rx_ = rx_.wrapping_add(rx);
        *tx_ = tx_.wrapping_add(tx);
        *bx_ = bx_.wrapping_add(bx);
        if dc {
            *dcs_ = dcs_.wrapping_add(1);
        }
    }

    pub fn dump(&mut self, clear: bool) -> proto::EpochDump {
        let mut v = Vec::new();

        for (k,(rx,tx,bx,dcs)) in self.v.iter() {
            v.push(proto::EpochDumpPacketCounter{
                identity: k.as_bytes().to_vec(),
                dcs:    *dcs,
                rx:     *rx,
                tx:     *tx,
                bx:     *bx,
            });
        }

        if clear {
            self.v.clear();
        }

        proto::EpochDump{
            packets: v
        }
    }
}
