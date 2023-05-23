mod common;
mod join;
mod race;
mod race_ok;
mod try_join;
mod utils;

pub fn join<F: join::Join>(f: F) -> F::Future {
    f.join()
}
pub fn race<F: race::Race>(f: F) -> F::Future {
    f.race()
}
pub fn try_join<F: try_join::TryJoin>(f: F) -> F::Future {
    f.try_join()
}

pub fn race_ok<F: race_ok::RaceOk>(f: F) -> F::Future {
    f.race_ok()
}

#[cfg(test)]
fn block_for_testing<F: core::future::Future>(f: F) -> F::Output {
    crate::DOM_CONTEXT.set(&crate::DomContext::Null, || {
        futures_lite::future::block_on(f)
    })
}
