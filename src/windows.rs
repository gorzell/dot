use std::ffi::CString;
use std::mem;
use std::ptr::{null, null_mut};
use winapi::shared::ntdef::LUID;
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::{AdjustTokenPrivileges, GetTokenInformation};
use winapi::um::winbase::LookupPrivilegeValueA;
use winapi::um::winnt::{
    TokenElevation, TokenElevationType, HANDLE, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_ELEVATION, TOKEN_PRIVILEGES, TOKEN_QUERY,
};

struct Handle(HANDLE);

impl Handle {
    fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

pub fn enable_privilege(name: &str) -> Result<(), &'static str> {
    let token = open_process_token(TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY)?;
    let luid = lookup_privilege_value(name)?;
    let mut privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };

    let success = unsafe {
        AdjustTokenPrivileges(
            token.as_raw(),
            0,
            &mut privileges,
            0,
            null_mut(),
            null_mut(),
        )
    };
    if success == 0 || unsafe { GetLastError() } != ERROR_SUCCESS {
        return Err("failed to adjust token privilege");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn is_elevated() -> Result<bool, &'static str> {
    let token = open_process_token(TOKEN_QUERY)?;
    let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut size = mem::size_of_val(&elevation) as u32;
    let success = unsafe {
        GetTokenInformation(
            token.as_raw(),
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        )
    };
    if success == 0 {
        return Err("failed to get token information");
    }
    Ok(elevation.TokenIsElevated != 0)
}

#[derive(Debug, PartialEq)]
pub enum ElevationType {
    Default = 1,
    Full,
    Limited,
}

pub fn get_elevation_type() -> Result<ElevationType, &'static str> {
    let token = open_process_token(TOKEN_QUERY)?;
    let mut elevation_type: u32 = 0;
    let mut size = mem::size_of_val(&elevation_type) as u32;
    let success = unsafe {
        GetTokenInformation(
            token.as_raw(),
            TokenElevationType,
            &mut elevation_type as *mut _ as *mut _,
            size,
            &mut size,
        )
    };
    if success == 0 {
        return Err("failed to get token information");
    }

    match elevation_type {
        1 => Ok(ElevationType::Default),
        2 => Ok(ElevationType::Full),
        3 => Ok(ElevationType::Limited),
        _ => Err("unknown elevation type"),
    }
}

fn open_process_token(token_type: u32) -> Result<Handle, &'static str> {
    let mut token = null_mut();
    let success = unsafe { OpenProcessToken(GetCurrentProcess(), token_type, &mut token) };
    if success == 0 {
        return Err("failed to get process token");
    }
    Ok(Handle(token))
}

fn lookup_privilege_value(name: &str) -> Result<LUID, &'static str> {
    let mut luid = LUID {
        LowPart: 0,
        HighPart: 0,
    };
    let name = CString::new(name).map_err(|_| "invalid privilege name")?;
    let success = unsafe { LookupPrivilegeValueA(null(), name.as_ptr(), &mut luid) };
    if success == 0 {
        return Err("failed to get the privilege value");
    }
    Ok(luid)
}

#[cfg(test)]
mod tests {
    use super::{enable_privilege, get_elevation_type};

    #[test]
    fn reads_current_process_elevation() {
        assert!(get_elevation_type().is_ok());
    }

    #[test]
    fn rejects_invalid_privilege_names() {
        assert!(enable_privilege("not-a-windows-privilege").is_err());
        assert_eq!(
            enable_privilege("invalid\0privilege"),
            Err("invalid privilege name")
        );
    }
}
