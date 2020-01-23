include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::GGError;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

const BUFFER_SIZE: usize = 512;

fn read_response_data(req_to_read: gg_request) -> Result<Vec<u8>, GGError> {
    let mut secret_bytes: Vec<u8> = Vec::new();

    unsafe {
        loop {
            let mut buffer = [0u8; BUFFER_SIZE];
            let mut read: usize = 0;
            let raw_read = &mut read as *mut usize;

            let read_res = gg_request_read(
                req_to_read,
                buffer.as_mut_ptr() as *mut c_void,
                BUFFER_SIZE,
                raw_read,
            );
            GGError::from_code(read_res)?;

            if read > 0 {
                secret_bytes.extend_from_slice(&buffer[..read]);
            } else {
                break;
            }
        }
    }

    Ok(secret_bytes)
}

/// Fetch the specified secrete from the green grass secret store
pub fn read_secret(secret_name: &str) -> Result<Vec<u8>, GGError> {
    unsafe {
        let mut req: gg_request = ptr::null_mut();
        let req_init = gg_request_init(&mut req);
        GGError::from_code(req_init)?;

        let secret_name_c = CString::new(secret_name).map_err(GGError::from)?;
        let mut res = gg_request_result {
            request_status: gg_request_status_GG_REQUEST_SUCCESS,
        };

        let fetch_res = gg_get_secret_value(
            req,
            secret_name_c.as_ptr(),
            ptr::null(),
            ptr::null(),
            &mut res,
        );
        GGError::from_code(fetch_res)?;

        let read_res = read_response_data(req)?;

        let close_res = gg_request_close(req);
        GGError::from_code(close_res)?;

        Ok(read_res)
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_nothing() {
        let secret_name  = "no_secret";
        match read_secret(secret_name) {
            Ok(bytes) => assert!(bytes.len() == 0),
            Err(code) => println!("Received error {}", code),
        }
    }
}
*/
