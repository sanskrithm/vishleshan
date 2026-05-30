/// Extended Dhatu system for Panini-RS (ASCII transliteration only).
///
/// Dhatus are semantic operation opcodes.
/// These are the fundamental semantic actions that can be composed.
///
/// All using ASCII transliteration (no Unicode diacritics):
/// - at (apādāna, source)
/// - ena (karaṇa, instrument)
/// - asya (output target)
/// - tva (lazy continuation, sūtra)
/// - ti (terminal execution, sūtra)
///
/// DHATU CATEGORIES:
/// 1. Aggregation dhatus (yuj, madh, gan, lagh, mah)
/// 2. Transformation dhatus (chid, adhyaya, kram, vibhaj)
/// 3. Relational dhatus (mel, etc.)
/// 4. Predicate dhatus (adhik, nyun, sam, asam)
/// 5. Terminal dhatus (drsh)
use std::fmt;

/// Aggregation operations (Samkhya dhatus)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AggregationDhatu {
    /// yuj: sum/aggregate
    Yuj,
    /// madh: mean/average
    Madh,
    /// gan: count
    Gan,
    /// lagh: minimum
    Lagh,
    /// mah: maximum
    Mah,
}

impl fmt::Display for AggregationDhatu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yuj => write!(f, "yuj"),
            Self::Madh => write!(f, "madh"),
            Self::Gan => write!(f, "gan"),
            Self::Lagh => write!(f, "lagh"),
            Self::Mah => write!(f, "mah"),
        }
    }
}

/// Transformation operations (Kriya dhatus)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformationDhatu {
    /// chid: filter
    Chid,
    /// adhyaya: project/select
    Adhyaya,
    /// kram: sort
    Kram,
    /// vibhaj: partition
    Vibhaj,
}

impl fmt::Display for TransformationDhatu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chid => write!(f, "chid"),
            Self::Adhyaya => write!(f, "adhyaya"),
            Self::Kram => write!(f, "kram"),
            Self::Vibhaj => write!(f, "vibhaj"),
        }
    }
}

/// Relational operations (Sambandha dhatus)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationalDhatu {
    /// ci: group
    Ci,
    /// mel: join
    Mel,
}

impl fmt::Display for RelationalDhatu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ci => write!(f, "ci"),
            Self::Mel => write!(f, "mel"),
        }
    }
}

/// Predicate operations (Niyama dhatus)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PredicateDhatu {
    /// adhik: greater than
    Adhik,
    /// nyun: less than
    Nyun,
    /// sam: equal
    Sam,
    /// asam: not equal
    Asam,
}

impl fmt::Display for PredicateDhatu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adhik => write!(f, "adhik"),
            Self::Nyun => write!(f, "nyun"),
            Self::Sam => write!(f, "sam"),
            Self::Asam => write!(f, "asam"),
        }
    }
}

/// Terminal/evaluation dhatus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalDhatu {
    /// drsh: render/print
    Drsh,
}

impl fmt::Display for TerminalDhatu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Drsh => write!(f, "drsh"),
        }
    }
}

/// The complete Dhatu system as a sum type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dhatu {
    Aggregation(AggregationDhatu),
    Transformation(TransformationDhatu),
    Relational(RelationalDhatu),
    Predicate(PredicateDhatu),
    Terminal(TerminalDhatu),
    /// Custom dhatus for extensibility
    Custom,
}

impl fmt::Display for Dhatu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aggregation(d) => write!(f, "{}", d),
            Self::Transformation(d) => write!(f, "{}", d),
            Self::Relational(d) => write!(f, "{}", d),
            Self::Predicate(d) => write!(f, "{}", d),
            Self::Terminal(d) => write!(f, "{}", d),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl Dhatu {
    /// Parse a dhatu from ASCII string
    pub fn from_ascii(s: &str) -> Option<Self> {
        match s {
            // Aggregation
            "yuj" => Some(Self::Aggregation(AggregationDhatu::Yuj)),
            "madh" => Some(Self::Aggregation(AggregationDhatu::Madh)),
            "gan" => Some(Self::Aggregation(AggregationDhatu::Gan)),
            "lagh" => Some(Self::Aggregation(AggregationDhatu::Lagh)),
            "mah" => Some(Self::Aggregation(AggregationDhatu::Mah)),

            // Transformation
            "chid" => Some(Self::Transformation(TransformationDhatu::Chid)),
            "adhyaya" => Some(Self::Transformation(TransformationDhatu::Adhyaya)),
            "kram" => Some(Self::Transformation(TransformationDhatu::Kram)),
            "vibhaj" => Some(Self::Transformation(TransformationDhatu::Vibhaj)),

            // Relational
            "ci" => Some(Self::Relational(RelationalDhatu::Ci)),
            "mel" => Some(Self::Relational(RelationalDhatu::Mel)),

            // Predicate
            "adhik" => Some(Self::Predicate(PredicateDhatu::Adhik)),
            "nyun" => Some(Self::Predicate(PredicateDhatu::Nyun)),
            "sam" => Some(Self::Predicate(PredicateDhatu::Sam)),
            "asam" => Some(Self::Predicate(PredicateDhatu::Asam)),

            // Terminal
            "drsh" => Some(Self::Terminal(TerminalDhatu::Drsh)),

            _ => None,
        }
    }

    /// Classify dhatu by type
    pub fn category(&self) -> &'static str {
        match self {
            Self::Aggregation(_) => "aggregation",
            Self::Transformation(_) => "transformation",
            Self::Relational(_) => "relational",
            Self::Predicate(_) => "predicate",
            Self::Terminal(_) => "terminal",
            Self::Custom => "custom",
        }
    }
}

/// Sūtra suffixes control execution semantics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SutraSuffix {
    /// tva: lazy continuation
    Tva,
    /// ti: terminal execution
    Ti,
}

impl fmt::Display for SutraSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tva => write!(f, "tva"),
            Self::Ti => write!(f, "ti"),
        }
    }
}

impl SutraSuffix {
    /// Parse from ASCII
    pub fn from_ascii(s: &str) -> Option<Self> {
        match s {
            "tva" => Some(Self::Tva),
            "ti" => Some(Self::Ti),
            _ => None,
        }
    }
}

/// Karaka case markers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KarakaSuffix {
    /// at: apādāna (source)
    At,
    /// ena: karaṇa (instrument)
    Ena,
    /// asya: karma (output target)
    Asya,
}

impl fmt::Display for KarakaSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::At => write!(f, "at"),
            Self::Ena => write!(f, "ena"),
            Self::Asya => write!(f, "asya"),
        }
    }
}

impl KarakaSuffix {
    /// Parse from ASCII
    pub fn from_ascii(s: &str) -> Option<Self> {
        match s {
            "at" => Some(Self::At),
            "ena" => Some(Self::Ena),
            "asya" => Some(Self::Asya),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_dhatus() {
        assert_eq!(
            Dhatu::from_ascii("yuj"),
            Some(Dhatu::Aggregation(AggregationDhatu::Yuj))
        );
        assert_eq!(
            Dhatu::from_ascii("madh"),
            Some(Dhatu::Aggregation(AggregationDhatu::Madh))
        );
        assert_eq!(
            Dhatu::from_ascii("gan"),
            Some(Dhatu::Aggregation(AggregationDhatu::Gan))
        );
    }

    #[test]
    fn test_transformation_dhatus() {
        assert_eq!(
            Dhatu::from_ascii("chid"),
            Some(Dhatu::Transformation(TransformationDhatu::Chid))
        );
        assert_eq!(
            Dhatu::from_ascii("adhyaya"),
            Some(Dhatu::Transformation(TransformationDhatu::Adhyaya))
        );
        assert_eq!(
            Dhatu::from_ascii("kram"),
            Some(Dhatu::Transformation(TransformationDhatu::Kram))
        );
    }

    #[test]
    fn test_sutra_suffixes() {
        assert_eq!(SutraSuffix::from_ascii("tva"), Some(SutraSuffix::Tva));
        assert_eq!(SutraSuffix::from_ascii("ti"), Some(SutraSuffix::Ti));
    }

    #[test]
    fn test_karaka_suffixes() {
        assert_eq!(KarakaSuffix::from_ascii("at"), Some(KarakaSuffix::At));
        assert_eq!(KarakaSuffix::from_ascii("ena"), Some(KarakaSuffix::Ena));
        assert_eq!(KarakaSuffix::from_ascii("asya"), Some(KarakaSuffix::Asya));
    }

    #[test]
    fn test_dhatu_categories() {
        let dhatu = Dhatu::Aggregation(AggregationDhatu::Yuj);
        assert_eq!(dhatu.category(), "aggregation");
    }
}
