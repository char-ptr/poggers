/// for external usage
#[feature(external)]
pub mod external;
/// for internal usage
#[feature(internal)]
pub mod internal;

pub(super) const WIN_PAGE_SIZE: usize = 0x1000;
