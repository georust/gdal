use std::{
    ffi::CString,
    sync::{Arc, Mutex},
};

use gdal::{config, errors::CplErrType};
use gdal_sys::{CPLErr, CPLError};

#[test]
fn test_error_handler() {
    // We cannot test different error handler scenarios in parallel since we modify a global error handler in GDAL.
    // Therefore, we test the error handler behavior sequentially to avoid data races.

    use_error_handler();

    error_handler_interleaved();
}

fn use_error_handler() {
    let errors: Arc<Mutex<Vec<(CplErrType, i32, String)>>> = Arc::new(Mutex::new(Vec::new()));

    let errors_clone = errors.clone();

    config::set_error_handler(move |a, b, c| {
        errors_clone.lock().unwrap().push((a, b, c.to_string()));
    });

    let msg = CString::new("foo".as_bytes()).unwrap();
    unsafe {
        CPLError(CPLErr::CE_Failure, 42, msg.as_ptr());
    };

    let msg = CString::new("bar".as_bytes()).unwrap();
    unsafe {
        CPLError(CPLErr::CE_Warning, 1, msg.as_ptr());
    };

    config::remove_error_handler();

    let result: Vec<(CplErrType, i32, String)> = errors.lock().unwrap().clone();
    assert_eq!(
        result,
        vec![
            (CplErrType::Failure, 42, "foo".to_string()),
            (CplErrType::Warning, 1, "bar".to_string())
        ]
    );
}

fn error_handler_interleaved() {
    use std::thread;
    // Two racing threads trying to set error handlers
    // First one
    thread::spawn(move || loop {
        config::set_error_handler(move |_a, _b, _c| {});
    });

    // Second one
    thread::spawn(move || loop {
        config::set_error_handler(move |_a, _b, _c| {});
    });

    // A thread that provokes potential race conditions
    let join_handle = thread::spawn(move || {
        for _ in 0..100 {
            let msg = CString::new("foo".as_bytes()).unwrap();
            unsafe {
                CPLError(CPLErr::CE_Failure, 42, msg.as_ptr());
            };

            let msg = CString::new("bar".as_bytes()).unwrap();
            unsafe {
                CPLError(CPLErr::CE_Warning, 1, msg.as_ptr());
            };
        }
    });

    join_handle.join().unwrap();
}
