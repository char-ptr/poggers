use windows::Win32::Foundation::CloseHandle;

struct Process {
    handl : windows::Win32::Foundation::HANDLE,
    pid: i32,
    name: String,
}

impl Process {
    fn new_from_pid(pid: i32) -> Process {
        Process {
            handl: windows::Win32::Foundation::HANDLE(0),
            pid,
            name,
        }
    }
    fn new_from_name(name: String) -> Process {
        Process {
            handl: windows::Win32::Foundation::HANDLE(0),
            pid: 0,
            name,
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handl);
        }
    }
}