use crate::structures::{
    create_snapshot,
    proc_list::{ProcList, ProcessList},
};

pub type PlatformData = super::create_snapshot::STProcess;

impl ProcList for ProcessList {
    fn get_iter() -> Result<
        impl Iterator<Item = crate::structures::proc_list::ProcessListEntry>,
        crate::structures::proc_list::ProcListError,
    > {
        Ok(create_snapshot::ToolSnapshot::new_process()
            .unwrap()
            .map(|i| crate::structures::proc_list::ProcessListEntry { pid: i.id, pd: i }))
    }
}
