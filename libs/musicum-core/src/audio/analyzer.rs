use crate::ProcessorChain;

pub struct ChainAnalyzer {
    context: AnalysisContext
}

impl ChainAnalyzer {

    // pub fn new(context: AnalysisContext) -> Self {
    //     ChainAnalyzer { context }
    // }

    pub fn run_analysis(&self, _chain: &ProcessorChain) -> anyhow::Result<()> {
        todo!("chain analysis pass not yet implemented")
    }
}
