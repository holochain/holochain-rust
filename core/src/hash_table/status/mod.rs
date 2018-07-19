// @TODO are these the correct key names?
// @see https://github.com/holochain/holochain-rust/issues/143
pub const STATUS_NAME: &str = "crud-status";
pub const LINK_NAME: &str = "crud-link";

bitflags! {
    #[derive(Default)]
    /// the CRUD status of a Pair is stored as PairMeta in the hash table, NOT in the pair itself
    /// statuses are represented as bitflags so we can easily build masks for filtering lookups
    pub struct CRUDStatus: u8 {
        const LIVE = 0x01;
        const REJECTED = 0x02;
        const DELETED = 0x04;
        const MODIFIED = 0x08;
        const ANY = 0xFF;
    }
}

#[cfg(test)]
mod tests {
    use super::CRUDStatus;

    #[test]
    /// test the CRUDStatus bit flags as ints
    fn status_bits() {
        assert_eq!(CRUDStatus::default().bits(), 0);
        assert_eq!(CRUDStatus::all().bits(), 255);

        assert_eq!(CRUDStatus::LIVE.bits(), 1);
        assert_eq!(CRUDStatus::REJECTED.bits(), 2);
        assert_eq!(CRUDStatus::DELETED.bits(), 4);
        assert_eq!(CRUDStatus::MODIFIED.bits(), 8);
        assert_eq!(CRUDStatus::ANY.bits(), 255);
    }

    #[test]
    /// test that we can build status masks from the CRUDStatus bit flags
    fn bitwise() {
        let example_mask = CRUDStatus::REJECTED | CRUDStatus::DELETED;
        assert!(example_mask.contains(CRUDStatus::REJECTED));
        assert!(example_mask.contains(CRUDStatus::DELETED));
        assert!(!example_mask.contains(CRUDStatus::LIVE));
        assert!(!example_mask.contains(CRUDStatus::MODIFIED));

        assert!(CRUDStatus::ANY.contains(CRUDStatus::LIVE));
    }
}
