mod types;
mod utils;

use std::pin::pin;

use types::*;
use utils::{is_all_pending, is_all_ready};
use x_bow::{PathExt, PathExtGuaranteed, Store};

#[pollster::test]
async fn regular_changes() {
    let state = Root::default();
    let state = Store::new(state);
    state
        .build_path()
        .field_1()
        .field_12()
        .borrow_mut()
        .push(());
    state
        .build_path()
        .field_2()
        .borrow_mut()
        .push(Enum2::VariantA(Default::default()));
    state
        .build_path()
        .field_2()
        .borrow_mut()
        .push(Enum2::VariantB {
            field: Default::default(),
        });

    let stream = state.build_path();
    let mut stream = pin!(stream.until_change());
    let stream_1 = state.build_path().field_1();
    let mut stream_1 = pin!(stream_1.until_change());
    let stream_1_11 = state.build_path().field_1().field_11();
    let mut stream_1_11 = pin!(stream_1_11.until_change());
    let stream_1_12 = state.build_path().field_1().field_12();
    let mut stream_1_12 = pin!(stream_1_12.until_change());
    let stream_2 = state.build_path().field_2();
    let mut stream_2 = pin!(stream_2.until_change());
    let stream_2_1 = state.build_path().field_2().index(1);
    let mut stream_2_1 = pin!(stream_2_1.until_change());
    let stream_2_1_a = state.build_path().field_2().index(1).VariantA_0();
    let mut stream_2_1_a = pin!(stream_2_1_a.until_change());
    let stream_2_1_b = state.build_path().field_2().index(1).VariantB_field();
    let mut stream_2_1_b = pin!(stream_2_1_b.until_change());
    let stream_2_1_b_d = state
        .build_path()
        .field_2()
        .index(1)
        .VariantB_field()
        .data();
    let mut stream_2_1_b_d = pin!(stream_2_1_b_d.until_change());

    assert!(
        is_all_pending([
            stream.as_mut(),
            stream_1.as_mut(),
            stream_1_11.as_mut(),
            stream_1_12.as_mut(),
            stream_2.as_mut(),
            stream_2_1.as_mut(),
            stream_2_1_a.as_mut(),
            stream_2_1_b.as_mut(),
            stream_2_1_b_d.as_mut(),
        ]),
        "all pending in the beginning"
    );

    state.build_path().field_2().borrow_mut();

    assert!(
        is_all_pending([
            stream.as_mut(),
            stream_1.as_mut(),
            stream_1_11.as_mut(),
            stream_1_12.as_mut(),
        ]),
        "field_1 not woken by field_2 change"
    );
    assert!(
        is_all_ready([
            stream_2.as_mut(),
            stream_2_1.as_mut(),
            stream_2_1_a.as_mut(),
            stream_2_1_b.as_mut(),
            stream_2_1_b_d.as_mut(),
        ]),
        "field_2 and descendants all woken"
    );
    assert!(
        is_all_pending([
            stream_2.as_mut(),
            stream_2_1.as_mut(),
            stream_2_1_a.as_mut(),
            stream_2_1_b.as_mut(),
            stream_2_1_b_d.as_mut(),
        ]),
        "woken once, pending later"
    );

    state.build_path().field_1().borrow_mut();
    assert!(
        is_all_ready([
            stream_1.as_mut(),
            stream_1_11.as_mut(),
            stream_1_12.as_mut(),
        ]),
        "field_1 and descendants all woken"
    );
    assert!(
        is_all_pending([
            stream.as_mut(),
            stream_2.as_mut(),
            stream_2_1.as_mut(),
            stream_2_1_a.as_mut(),
            stream_2_1_b.as_mut(),
            stream_2_1_b_d.as_mut(),
        ]),
        "field_2 not woken by field_1 change"
    );

    state.build_path().borrow_mut();
    assert!(
        is_all_ready([
            stream.as_mut(),
            stream_1.as_mut(),
            stream_1_11.as_mut(),
            stream_1_12.as_mut(),
            stream_2.as_mut(),
            stream_2_1.as_mut(),
            stream_2_1_a.as_mut(),
            stream_2_1_b.as_mut(),
            stream_2_1_b_d.as_mut(),
        ]),
        "all ready after root change"
    );

    {
        let deep_stream = state.build_path().field_1().field_12().index(0);
        let mut deep_stream = pin!(deep_stream.until_change());
        assert!(
            is_all_pending([deep_stream.as_mut()]),
            "deep stream starts off pending"
        );
        state
            .build_path()
            .field_1()
            .field_12()
            .index(0)
            .borrow_opt_mut();
        assert!(
            is_all_pending([
                stream.as_mut(),
                stream_1.as_mut(),
                stream_1_11.as_mut(),
                stream_1_12.as_mut(),
                stream_2.as_mut(),
                stream_2_1.as_mut(),
                stream_2_1_a.as_mut(),
                stream_2_1_b.as_mut(),
                stream_2_1_b_d.as_mut(),
            ]),
            "deep change wakes no one above"
        );
        assert!(
            is_all_ready([deep_stream.as_mut()]),
            "the deep change is woken"
        );
    }

    {
        let deep_stream = state.build_path().field_1().field_12().index(1);
        let mut deep_stream = pin!(deep_stream.until_change());
        assert!(
            is_all_pending([deep_stream.as_mut()]),
            "deep stream starts off pending"
        );
        state
            .build_path()
            .field_1()
            .field_12()
            .index(1)
            .borrow_opt_mut();
        assert!(
            is_all_pending([
                stream.as_mut(),
                stream_1.as_mut(),
                stream_1_11.as_mut(),
                stream_1_12.as_mut(),
                stream_2.as_mut(),
                stream_2_1.as_mut(),
                stream_2_1_a.as_mut(),
                stream_2_1_b.as_mut(),
                stream_2_1_b_d.as_mut(),
            ]),
            "deep change wakes no one above"
        );
        assert!(
            is_all_pending([deep_stream.as_mut()]),
            "the deep change is not woken because borrow_opt_mut failed"
        );
    }
}
