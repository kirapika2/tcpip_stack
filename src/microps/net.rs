use crate::Platform;

pub fn net_init() -> Result<(), i32> {
    crate::log_info!("initialize...");
    match Platform::init() {
        Ok(_) => {
            crate::log_info!("success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("platform_init() failed: {}", e);
            Err(e)
        }
    }
}

pub fn net_run() -> Result<(), i32> {
    crate::log_info!("startup...");
    match Platform::run() {
        Ok(_) => {
            crate::log_info!("success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("platform_run() failed: {}", e);
            Err(e)
        }
    }
}

pub fn net_shutdown() -> Result<(), i32> {
    crate::log_info!("shutdown...");
    match Platform::shutdown() {
        Ok(_) => {
            crate::log_info!("success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("platform_shutdown() failed: {}", e);
            Err(e)
        }
    }
}
