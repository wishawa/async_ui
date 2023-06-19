use x_bow::create_store;
use x_bow::Trackable;
use x_bow::Tracked;
use x_bow::TrackedGuaranteed;

#[test]
fn just_leaf() {
    let a: i32 = 5;
    let store = create_store(a);
    let _ = async {
        store.until_change().await;
    };
}

#[test]
fn struct_project() {
    #[derive(Trackable, Default)]
    #[track(deep)]
    struct Struct1 {
        field_1: Struct2,
        field_2: i32,
        field_3: Struct3,
    }
    #[derive(Trackable, Default)]
    struct Struct2 {
        field_1: String,
    }
    #[derive(Trackable, Default)]
    struct Struct3(u64, String);

    let store = create_store(Struct1::default());
    {
        let _: &Struct2 = &*store.field_1.borrow();
    }
    {
        let _: &String = &*store.field_1.field_1.borrow();
    }
    {
        let _: &i32 = &*store.field_2.borrow();
    }
    {
        let _: &Struct3 = &*store.field_3.borrow();
    }
    {
        let _: &u64 = &*store.field_3.t0.borrow();
    }
    {
        let _: &String = &*store.field_3.t1.borrow();
    }
    let _ = async {
        store.field_1.until_change().await;
        store.field_1.field_1.until_change().await;
        store.field_2.until_change().await;
        store.field_3.until_change().await;
        store.field_3.t0.until_change().await;
        store.field_3.t1.until_change().await;
    };
}

#[test]
fn generic_struct_project() {
    #[derive(Trackable)]
    #[track(deep)]
    struct MyStruct<T> {
        field: T,
    }

    trait HasAssoc {
        type AssocType;
    }
    #[derive(Trackable)]
    #[track(deep)]
    struct MyStructWithAssoc<T: HasAssoc> {
        field: T::AssocType,
    }
    impl HasAssoc for i32 {
        type AssocType = String;
    }

    let store = create_store(MyStructWithAssoc::<i32> {
        field: "hello".into(),
    });
    {
        let _: &MyStructWithAssoc<_> = &*store.borrow();
    }
    {
        let _: &String = &*store.field.borrow();
    }
}

#[test]
fn enum_project() {
    #[derive(Trackable)]
    enum MyEnum<T> {
        Variant1(i16, i32, i64),
        Variant2 { first: i16, second: i32, third: T },
    }
    let store = create_store(MyEnum::Variant1(1, 2, 3));
    {
        let _: Option<std::cell::Ref<'_, i32>> = store.Variant2_second.borrow_opt();
    }
    {
        let _: Option<std::cell::Ref<'_, i16>> = store.Variant1_0.borrow_opt();
    }
    *store.borrow_mut() = MyEnum::Variant2 {
        first: 5,
        second: 6,
        third: String::new(),
    };
}

#[test]
fn guarantee() {
    #[derive(Trackable)]
    #[track(deep)]
    enum MyEnum {
        Var1(isize),
        Var2(Option<()>),
        Var3(MyStruct),
    }
    #[derive(Trackable)]
    #[track(deep)]
    struct MyStruct {
        field: usize,
    }
    let store = create_store(MyEnum::Var1(123));
    store.borrow();
    // store.Var3_0.field.borrow();
}
