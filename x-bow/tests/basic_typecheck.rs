use x_bow::create_store;
use x_bow::Trackable;
use x_bow::Tracked;
use x_bow::TrackedGuaranteed;

#[test]
fn just_leaf() {
    let mut a: i32 = 5;
    let store = create_store();
    let store = store.initialize(&mut a);
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

    let mut data = Struct1::default();
    let store = create_store();
    let store = store.initialize(&mut data);
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

    let mut data = MyStructWithAssoc::<i32> {
        field: "hello".into(),
    };
    let store = create_store();
    let store = store.initialize(&mut data);
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
    let mut data = MyEnum::Variant1(1, 2, 3);
    let store = create_store();
    let store = store.initialize(&mut data);
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
        _Var2(Option<()>),
        _Var3(MyStruct),
    }
    #[derive(Trackable)]
    #[track(deep)]
    struct MyStruct {
        field: usize,
    }
    let mut data = MyEnum::Var1(123);
    let store = create_store();
    let store = store.initialize(&mut data);
    store.borrow();
    // store.Var3_0.field.borrow();
}

#[test]
fn tuple() {
    let mut data = (3,);
    let store = create_store();
    let store = store.initialize(&mut data);
    let _b = store.borrow();
}
