use x_bow::{PathExtGuaranteed, Store};
use x_bow_macros::Trackable;

#[test]
fn get_and_set() {
    #[derive(Trackable, Debug, Default, PartialEq, Clone)]
    #[track(deep)]
    struct State {
        a: String,
        b: (Struct1, i32),
    }
    #[derive(Trackable, Debug, Default, PartialEq, Clone)]
    #[track(deep)]
    struct Struct1 {
        c: i32,
    }
    let state = Store::new(State {
        a: String::from("Hello World"),
        b: (Struct1 { c: 5678 }, 1234),
    });
    assert_eq!(state.build_path().b().t1().get(), 1234);
    assert_eq!(state.build_path().b().t0().get(), Struct1 { c: 5678 });
    assert_eq!(state.build_path().b().t0().c().get(), 5678);
    state.build_path().b().t0().set(Struct1 { c: 42 });
    assert_eq!(state.build_path().b().t0().c().get(), 42);
}
