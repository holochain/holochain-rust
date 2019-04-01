#[macro_use]
extern crate bencher;
extern crate holochain_cas_implementations;
extern crate holochain_core_types;
extern crate tempfile;

use self::tempfile::tempdir;
use bencher::Bencher;
use holochain_cas_implementations::eav::{file::EavFileStorage,pickle::EavPickleStorage};
use holochain_core_types::{
    cas::{
        content::{ExampleAddressableContent},
        storage::EavTestSuite,
    },

};
fn bench_file_eav_one_to_many(b: &mut Bencher) {
    b.iter(|| {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_one_to_many::<ExampleAddressableContent, EavFileStorage>(
            eav_storage.clone(),
        )
    })
}

fn bench_file_eav_many_to_one(b: &mut Bencher) {
    b.iter(|| {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_many_to_one::<ExampleAddressableContent, EavFileStorage>(
            eav_storage.clone(),
        )
    })
}


fn bench_pickle_eav_one_to_many(b: &mut Bencher) {
    b.iter(|| {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavPickleStorage::new(temp_path);
        EavTestSuite::test_one_to_many::<ExampleAddressableContent, EavPickleStorage>(
            eav_storage.clone(),
        )
    })
}

fn bench_pickle_eav_many_to_one(b: &mut Bencher) {
    b.iter(|| {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavPickleStorage::new(temp_path);
        EavTestSuite::test_many_to_one::<ExampleAddressableContent, EavPickleStorage>(
            eav_storage.clone(),
        )
    })
}

benchmark_group!(
    benches,
    bench_file_eav_one_to_many,
    bench_file_eav_many_to_one,
    bench_pickle_eav_many_to_one,
    bench_pickle_eav_one_to_many

);
benchmark_main!(benches);
