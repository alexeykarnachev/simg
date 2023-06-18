use std::cell::RefCell;
use std::os::raw::{c_double, c_int, c_uchar};

#[allow(non_camel_case_types)]
type em_callback_func = unsafe extern "C" fn();

extern "C" {
    // This extern is built in by Emscripten.
    pub fn emscripten_run_script_int(x: *const c_uchar) -> c_int;
    pub fn emscripten_cancel_main_loop();
    pub fn emscripten_get_now() -> c_double;
    pub fn emscripten_set_main_loop(
        func: em_callback_func,
        fps: c_int,
        simulate_infinite_loop: c_int,
    );
}

thread_local!(static MAIN_LOOP_CALLBACK: RefCell<Option<Box<dyn FnMut()>>> = RefCell::new(None));

pub fn set_main_loop_callback<F: 'static>(callback: F)
where
    F: FnMut(),
{
    MAIN_LOOP_CALLBACK.with(|log| {
        *log.borrow_mut() = Some(Box::new(callback));
    });

    unsafe {
        emscripten_set_main_loop(wrapper::<F>, 0, 1);
    }

    extern "C" fn wrapper<F>()
    where
        F: FnMut(),
    {
        MAIN_LOOP_CALLBACK.with(|z| {
            if let Some(ref mut callback) = *z.borrow_mut() {
                callback();
            }
        });
    }
}
