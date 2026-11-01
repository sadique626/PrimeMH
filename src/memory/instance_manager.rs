extern crate winapi;

use winapi::shared::windef::HWND;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
use winapi::um::winuser::{EnumWindows,  GetWindowInfo, GetWindowTextW, WINDOWINFO};
use winapi::um::psapi::{EnumProcesses, GetModuleFileNameExW};
use winapi::um::winuser::GetWindowThreadProcessId;
use winapi::shared::minwindef::{DWORD, LPARAM};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;


#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub pid: DWORD,
    pub hwnd: HWND,
    pub title: String,
}

pub fn get_process_pid_and_window_handle() -> Vec<WindowInfo> {
    let pids = get_d2r_process_pids();
    let windows = get_window_handles_and_titles_for_pids(pids);
    return windows;
}


fn get_d2r_process_pids() -> Vec<DWORD> {
    let mut pids: Vec<DWORD> = Vec::new();
    let mut processes: [DWORD; 1024] = [0; 1024];
    let mut cb_needed: DWORD = 0;

    unsafe {
        // Get the list of process IDs
        if EnumProcesses(processes.as_mut_ptr(), std::mem::size_of_val(&processes) as DWORD, &mut cb_needed) != 0 {
            let num_processes = cb_needed / std::mem::size_of::<DWORD>() as DWORD;

            for i in 0..num_processes {
                let pid = processes[i as usize];
                if pid == 0 {
                    continue;
                }

                // Check the process name for each PID
                let process_name = get_process_name(pid);
                if is_game_exe(&process_name) {
                    log::info!("Found D2R.exe process with PID: {} {:?}", pid, process_name);
                    pids.push(pid);
                }
            }
        }
    }
    log::info!("Found {} D2R.exe / D2RLoader.exe processes", pids.len());
    pids
}

/// D2R.exe (retail) or D2RLoader.exe (D2RLoader / D2R Reimagined)
pub fn is_game_exe(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with("\\d2r.exe") || lower.ends_with("\\d2rloader.exe") || lower == "d2r.exe" || lower == "d2rloader.exe"
}

pub fn is_loader_pid(pid: DWORD) -> bool {
    get_process_name(pid).to_lowercase().ends_with("d2rloader.exe")
}

pub fn get_process_name(pid: DWORD) -> String {
    unsafe {
        let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
        
        if process_handle.is_null() {
            return String::new();
        }

        let mut filename = vec![0u16; 1024];
        let len = 1024;
        if GetModuleFileNameExW(process_handle, std::ptr::null_mut(), filename.as_mut_ptr(), len as DWORD) > 0 {
            let os_string = OsString::from_wide(&filename);
            os_string.to_string_lossy().trim_end_matches('\0').to_string()
        } else {
            let error_code = GetLastError();
            log::debug!("GetModuleFileNameExW failed with error: {}", error_code);
            String::new()
        }
    }
}


fn get_window_handles_and_titles_for_pids(pids: Vec<DWORD>) -> Vec<WindowInfo> {
    let mut window_info_list: Vec<WindowInfo> = Vec::new();
    unsafe {
        EnumWindows(Some(enum_window_proc), &mut window_info_list as *mut _ as LPARAM);
    }

    window_info_list.into_iter().filter(|window| {
        pids.contains(&get_process_pid(window.hwnd)) && is_window_visible(window.hwnd)
    }).collect()
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, l_param: LPARAM) -> i32 {
    let window_info_list = &mut *(l_param as *mut Vec<WindowInfo>);
    let window_title = get_window_title(hwnd);
    let window_info = WindowInfo { hwnd, title: window_title, pid: get_process_pid(hwnd) };
    window_info_list.push(window_info);
    1 
}

fn get_process_pid(hwnd: HWND) -> DWORD {
    let mut process_id = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut process_id);
    }
    process_id
}

fn get_window_title(hwnd: HWND) -> String {
    let mut title = vec![0u16; 256];
    unsafe {
        // Get the window title
        GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
    }
    let os_string = OsString::from_wide(&title);
    // log::info!("Found window with title: {:?}", os_string.to_string_lossy().trim_end_matches('\0'));
    os_string.to_string_lossy().trim_end_matches('\0').to_string()
}

fn is_window_visible(hwnd: HWND) -> bool {
    let mut window_info: WINDOWINFO = unsafe { std::mem::zeroed() };
    window_info.cbSize = std::mem::size_of::<WINDOWINFO>() as u32;
    unsafe {
        if GetWindowInfo(hwnd, &mut window_info) != 0 {
            return !(window_info.dwStyle & 0x10000000 == 0); // 0x10000000 is WS_VISIBLE
        }
    }
    false
}