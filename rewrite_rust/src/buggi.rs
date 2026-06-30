#[cfg(windows)]
use std::mem;
#[cfg(windows)]
use windows::Win32::System::Diagnostics::Debug::{CheckRemoteDebuggerPresent, IsDebuggerPresent};
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
#[cfg(windows)]
use windows::Win32::System::Threading::GetCurrentProcess;

#[cfg(windows)]
static BLACKLIST: &[&str] = &[
    "ollydbg",
    "ida",
    "ida64",
    "idag",
    "idag64",
    "idaw",
    "idaw64",
    "idaq",
    "idaq64",
    "idau",
    "idau64",
    "scylla",
    "protection_id",
    "x64dbg",
    "x32dbg",
    "windbg",
    "reshacker",
    "importrec",
    "immunitydebugger",
    "megadumper",
    "cheatengine",
    "cheat engine",
    "wireshark",
    "dumpcap",
    "tshark",
    "rawshark",
    "httpdebugger",
    "httpdebuggerpro",
    "fiddler",
    "charles",
    "processhacker",
    "process hacker",
    "procmon",
    "procexp",
    "procexp64",
    "dnspy",
    "de4dot",
    "ilspy",
    "dotpeek",
    "ghidra",
    "hxd",
    "pestudio",
    "autoruns",
    "burpsuite",
    "proxifier",
    "tcpdump",
    "reclass",
    "simpleassemblyexplorer",
    "ildasm",
    "graywolf",
    "mdbg",
    "regmon",
    "filemon",
    "lordpe",
    "sysinternals",
    "apitrace",
    "apimonitor",
    "api monitor",
];

#[cfg(windows)]
fn is_debugger_present() -> bool {
    unsafe { IsDebuggerPresent().as_bool() }
}

#[cfg(windows)]
fn check_remote_debugger() -> bool {
    unsafe {
        let handle = GetCurrentProcess();
        let mut debugger_present = windows::core::BOOL(0);
        if CheckRemoteDebuggerPresent(handle, &mut debugger_present).is_ok() {
            debugger_present.as_bool()
        } else {
            false
        }
    }
}

#[cfg(windows)]
fn check_process() -> Option<String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry = PROCESSENTRY32 {
            dwSize: mem::size_of::<PROCESSENTRY32>() as u32,
            ..mem::zeroed()
        };

        if Process32First(snapshot, &mut entry).is_err() {
            let _ = windows::Win32::Foundation::CloseHandle(snapshot);
            return None;
        }

        loop {
            let name_bytes: Vec<u8> = entry
                .szExeFile
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| c as u8)
                .collect();
            let name = String::from_utf8_lossy(&name_bytes).to_lowercase();

            for bl in BLACKLIST {
                if name.contains(bl) {
                    let _ = windows::Win32::Foundation::CloseHandle(snapshot);
                    return Some(name);
                }
            }

            if Process32Next(snapshot, &mut entry).is_err() {
                break;
            }
        }

        let _ = windows::Win32::Foundation::CloseHandle(snapshot);
        None
    }
}

#[cfg(windows)]
pub fn detect_and_return() -> Option<String> {
    if is_debugger_present() {
        return Some("debugger (IsDebuggerPresent)".into());
    }
    if check_remote_debugger() {
        return Some("remote debugger".into());
    }
    check_process()
}

#[cfg(windows)]
pub fn simple_run(delay_secs: u64) {
    std::thread::spawn(move || loop {
        if let Some(_) = detect_and_return() {
            std::process::exit(0);
        }
        std::thread::sleep(std::time::Duration::from_secs(delay_secs));
    });
}

#[cfg(not(windows))]
pub fn detect_and_return() -> Option<String> {
    None
}

#[cfg(not(windows))]
pub fn simple_run(_delay_secs: u64) {}
