use criterion::{black_box, criterion_group, criterion_main, Criterion};

struct SomeStruct<T> {
    value: T,
}

fn use_string() {
    let s = String::from("hello");
    let some_struct = SomeStruct { value: s };
    let _another_value = some_struct.value;
    black_box(_another_value);
}

fn use_string_with_clone() {
    let s = String::from("hello");
    let some_struct = SomeStruct { value: s.clone() };
    let _another_value = some_struct.value.clone();
    black_box(_another_value);
}

fn use_str() {
    let s = "hello";
    let some_struct = SomeStruct { value: s };
    let _another_value = some_struct.value;
    black_box(_another_value);
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("use only string", |b| b.iter(|| use_string()));
    c.bench_function("use only str", |b| b.iter(|| use_str()));
    c.bench_function("use string with clone", |b| {
        b.iter(|| use_string_with_clone())
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);

