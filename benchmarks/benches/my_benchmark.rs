#[macro_use]
extern crate bencher;
extern crate holochain_core_types;
extern crate lib3h_persistence_file;
extern crate lib3h_persistence_mem;
extern crate lib3h_persistence_pickle;
extern crate tempfile;

use self::tempfile::tempdir;
use bencher::Bencher;
use persistence_file::eav::file::EavFileStorage;

use persistence_api::cas::{content::ExampleAddressableContent, storage::EavTestSuite};
use persistence_mem::eav::memory::EavMemoryStorage;
use persistence_pickle::eav::pickle::EavPickleStorage;

fn bench_memory_eav_one_to_many(b: &mut Bencher) {
    b.iter(|| {
        let eav_storage = EavMemoryStorage::new();
        EavTestSuite::test_one_to_many::<ExampleAddressableContent, EavMemoryStorage>(
            eav_storage.clone(),
        )
    })
}

fn bench_memory_eav_many_to_one(b: &mut Bencher) {
    b.iter(|| {
        let eav_storage = EavMemoryStorage::new();
        EavTestSuite::test_one_to_many::<ExampleAddressableContent, EavMemoryStorage>(
            eav_storage.clone(),
        )
    })
}
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
    bench_memory_eav_many_to_one,
    bench_memory_eav_one_to_many,
    bench_file_eav_one_to_many,
    bench_file_eav_many_to_one,
    bench_pickle_eav_many_to_one,
    bench_pickle_eav_one_to_many
);
benchmark_main!(benches);
