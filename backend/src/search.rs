#![allow(dead_code)]

#[cfg(feature = "search")]
pub mod __search_impl {
    use anyhow::Result;
    use std::path::Path;
    use tantivy::collector::TopDocs;
    use tantivy::query::QueryParser;
    use tantivy::schema::{Field, Schema, FAST, STORED, STRING, TEXT};
    use tantivy::{doc, Index, IndexWriter};

    #[derive(Clone)]
    pub struct Fields {
        pub log_id: Field,
        pub timestamp: Field,
        pub speaker: Field,
        pub content: Field,
        pub embedding: Field,
    }

    pub struct SearchIndex {
        pub index: Index,
        pub schema: Schema,
        pub fields: Fields,
    }

    impl SearchIndex {
        pub fn create<P: AsRef<Path>>(dir: P) -> Result<Self> {
            let mut schema_builder = Schema::builder();

            let log_id = schema_builder.add_u64_field("log_id", FAST | STORED);
            let timestamp = schema_builder.add_i64_field("timestamp", FAST | STORED);
            let speaker = schema_builder.add_text_field("speaker", STRING | STORED);
            let content = schema_builder.add_text_field("content", TEXT);
            let embedding = schema_builder.add_bytes_field("embedding", STORED);

            let schema = schema_builder.build();

            let path = dir.as_ref();
            std::fs::create_dir_all(path)?;
            let index = Index::create_in_dir(path, schema.clone())?;

            Ok(Self {
                index,
                schema: schema.clone(),
                fields: Fields {
                    log_id,
                    timestamp,
                    speaker,
                    content,
                    embedding,
                },
            })
        }

        pub fn writer(&self, heap_size_bytes: usize) -> Result<IndexWriter> {
            Ok(self.index.writer(heap_size_bytes)?)
        }

        pub fn add_document(
            &self,
            writer: &mut IndexWriter,
            log_id_val: u64,
            timestamp_val: i64,
            speaker_val: &str,
            content_val: &str,
            embedding_bytes: Option<&[u8]>,
        ) -> Result<()> {
            let mut document = doc! {
                self.fields.log_id => log_id_val,
                self.fields.timestamp => timestamp_val,
                self.fields.speaker => speaker_val.to_string(),
                self.fields.content => content_val.to_string(),
            };
            if let Some(b) = embedding_bytes {
                document.add_bytes(self.fields.embedding, b.to_vec());
            }
            let _ = writer.add_document(document);
            Ok(())
        }

        pub fn search_content(&self, query_str: &str, limit: usize) -> Result<Vec<u64>> {
            let reader = self.index.reader()?;
            let searcher = reader.searcher();
            let qp = QueryParser::for_index(&self.index, vec![self.fields.content]);
            let query = qp.parse_query(query_str)?;
            let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

            let mut out = Vec::new();
            for (_score, _doc_address) in top_docs {
                out.push(1u64);
            }
            Ok(out)
        }
    }
}

#[cfg(feature = "search")]
pub use __search_impl::{Fields, SearchIndex};
