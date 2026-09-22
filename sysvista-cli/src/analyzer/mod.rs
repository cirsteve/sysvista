pub mod contract;
pub mod merge;
pub mod spawn;

pub use contract::{AnalyzeRequest, AnalyzeResponse, CONTRACT_VERSION};
pub use merge::{MergedAnalysis, merge};
pub use spawn::{AnalyzerError, analyze, handshake};
