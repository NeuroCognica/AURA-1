use aura_backend::search::SearchIndex;
use tempfile::TempDir;

#[test]
fn index_and_search_content() {
    let tmp = TempDir::new().expect("tempdir");
    let idx = SearchIndex::create(tmp.path()).expect("create index");

    let mut writer = idx.writer(50_000_000).expect("writer");
    idx.add_document(&mut writer, 1u64, 12345i64, "alice", "navigate to home", None)
        .expect("add doc");
    idx.add_document(&mut writer, 2u64, 12346i64, "bob", "search for navigation", None)
        .expect("add doc");

    writer.commit().expect("commit");

    let hits = idx.search_content("navigation", 10).expect("search");
    assert!(!hits.is_empty());
}
