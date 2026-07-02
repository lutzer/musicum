use crate::ProcessorChain;

pub struct ChainAnalyzer;

impl ChainAnalyzer {
    pub fn run_analysis(&self, _chain: &ProcessorChain) -> anyhow::Result<()> {
        todo!("chain analysis pass not yet implemented")
    }
}
