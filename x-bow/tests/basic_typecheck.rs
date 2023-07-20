use x_bow::PathExt;
use x_bow::PathExtGuaranteed;
use x_bow::Store;
use x_bow::Trackable;

#[test]
fn just_leaf() {
    let mut a: i32 = 5;
    let store = Store::new(a);
    let _ = async {
        let _ = store.build_path().until_change();
    };
}

#[test]
fn struct_project() {
    #[derive(Trackable, Default, Clone)]
    #[track(deep)]
    struct Struct1 {
        field_1: Struct2,
        field_2: i32,
        field_3: Struct3,
    }
    #[derive(Trackable, Default, Clone)]
    struct Struct2 {
        field_1: String,
    }
    #[derive(Trackable, Default, Clone)]
    struct Struct3(u64, String);

    let data = Struct1::default();
    let store = Store::new(data);
    {
        let _: Struct2 = store.build_path().field_1().borrow().clone();
    }
    {
        let _: String = store.build_path().field_1().field_1().borrow().clone();
    }
    {
        let _: i32 = store.build_path().field_2().borrow().clone();
    }
    {
        let _: Struct3 = store.build_path().field_3().borrow().clone();
    }
    {
        let _: u64 = store.build_path().field_3().t0().borrow().clone();
    }
    {
        let _: String = store.build_path().field_3().t1().borrow().clone();
    }
    let _ = async {
        store.build_path().field_1().until_change();
        store.build_path().field_1().field_1().until_change();
        store.build_path().field_2().until_change();
        store.build_path().field_3().until_change();
        store.build_path().field_3().t0().until_change();
        store.build_path().field_3().t1().until_change();
    };
}

#[test]
fn generic_struct_project() {
    #[derive(Trackable, Clone)]
    #[track(deep)]
    struct MyStruct<T> {
        field: T,
    }

    trait HasAssoc {
        type AssocType;
    }
    #[derive(Trackable, Clone)]
    #[track(deep)]
    #[x_bow(bound = "T::AssocType: Trackable")]
    struct MyStructWithAssoc<T: HasAssoc> {
        field: T::AssocType,
    }
    impl HasAssoc for i32 {
        type AssocType = String;
    }

    let data = MyStructWithAssoc::<i32> {
        field: "hello".into(),
    };
    let store = Store::new(data);
    {
        let _: MyStructWithAssoc<_> = store.build_path().borrow().clone();
    }
    {
        let _: String = store.build_path().field().borrow_mut().to_string();
    }
}

#[test]
fn enum_project() {
    #[derive(Trackable)]
    enum MyEnum<T> {
        Variant1(i16, i32, i64),
        Variant2 { first: i16, second: i32, third: T },
    }
    let data = MyEnum::Variant1(1, 2, 3);
    let store = Store::new(data);
    {
        let _: Option<std::cell::Ref<'_, i32>> = store.build_path().Variant2_second().borrow_opt();
    }
    {
        let _: Option<std::cell::Ref<'_, i16>> = store.build_path().Variant2_first().borrow_opt();
    }
    *store.build_path().borrow_mut() = MyEnum::Variant2 {
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
    let data = MyEnum::Var1(123);
    let store = Store::new(data);
    store.build_path().borrow();
    // store.Var3_0.field.borrow();
}

#[test]
fn tuple() {
    let data = (3,);
    let store = Store::new(data);
    let _b = store.build_path().borrow().clone();
}
