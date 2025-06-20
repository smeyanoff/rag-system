pub mod document;
pub use document::{Chunk, ChunkRepo, Document, DocumentPreparer, DocumentRepo};
pub mod embeddings;
pub use embeddings::{ChunkEmbending, ChunkEmbendingRepo, TextVectorizer, VectorSearcher};
