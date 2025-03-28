//! This module exposes options which can be set on a `Btor` instance.

pub enum ModelGen {
    /// Do not generate models
    Disabled,
    /// Generate models for asserted expressions only
    Asserted,
    /// Generate models for all expressions
    All,
}

#[repr(u64)]
pub enum LogLevel {
    Off = 0,
    Err = 1,
    Warn = 2,
    Debug = 3,
    Trace = 4,
}

#[repr(u64)]
pub enum Verbosity {
    None = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
}

pub enum SolverEngine {
    /// Default engine for all combinations of QF_AUFBV; uses lemmas on demand
    /// for QF_AUFBV and eager bit-blasting for QF_BV
    Fun,
    /// Score-based local search QF_BV engine
    SLS,
    /// Propagation-based local search QF_BV engine
    Prop,
    /// Propagation-based local search QF_BV engine that operates on the bit-blasted formula (the AIG layer)
    AIGProp,
    /// Quantifier engine (BV only)
    Quant,
}

pub enum SatEngine {
    /// CaDiCaL
    CaDiCaL,
    /// CryptoMiniSat
    CMS,
    /// Gimsatul
    Gimsatul,
    /// Kissat
    Kissat,
    /// Lingeling
    Lingeling,
    /// MiniSAT
    MiniSAT,
    /// PicoSAT
    PicoSAT,
}

pub enum RewriteLevel {
    /// "no rewriting"
    None,
    /// "term level rewriting"
    TermLevel,
    /// "more simplification techniques"
    More,
    /// "full rewriting/simplification"
    Full,
}
