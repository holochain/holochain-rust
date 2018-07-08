
bitflags! {
    #[derive(Default)]
    pub struct StatusMask: u8 {
        const LIVE = 0x01;
        const REJECTED = 0x02;
        const DELETED = 0x04;
        const MODIFIED = 0x08;
        const ANY = 0xFF;
    }
}

#[cfg(test)]
mod tests {
    use super::StatusMask;

    #[test]
    fn status_bits() {
        assert_eq!(StatusMask::default().bits(), 0);
        assert_eq!(StatusMask::all().bits(), 255);

        assert_eq!(StatusMask::LIVE.bits(), 1);
        assert_eq!(StatusMask::REJECTED.bits(), 2);
        assert_eq!(StatusMask::DELETED.bits(), 4);
        assert_eq!(StatusMask::MODIFIED.bits(), 8);
        assert_eq!(StatusMask::ANY.bits(), 255);
    }

    #[test]
    fn bitwise() {
        let example_mask = StatusMask::REJECTED | StatusMask::DELETED;
        assert!(example_mask.contains(StatusMask::REJECTED));
        assert!(example_mask.contains(StatusMask::DELETED));
        assert!(!example_mask.contains(StatusMask::LIVE));
        assert!(!example_mask.contains(StatusMask::MODIFIED));

        assert!(StatusMask::ANY.contains(StatusMask::LIVE));
    }
}
