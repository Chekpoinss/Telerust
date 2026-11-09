//! Research status for AyuGram compatibility. These are not claims of support.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compatibility {
    CandidateViaTdlib,
    ClientSideCandidate,
    ProhibitedByCurrentTelegramTerms,
    ResearchRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureNote {
    pub name: &'static str,
    pub status: Compatibility,
    pub evidence: &'static str,
}

pub const AYUGRAM_FEATURE_NOTES: &[FeatureNote] = &[
    FeatureNote {
        name: "Ghost Mode / suppress read or online state",
        status: Compatibility::ProhibitedByCurrentTelegramTerms,
        evidence: "Telegram API Terms §1.4",
    },
    FeatureNote {
        name: "Anti-Recall / retain deleted content",
        status: Compatibility::ResearchRequired,
        evidence: "AyuGram feature list; behavior and API constraints need source-level review",
    },
    FeatureNote {
        name: "Font and appearance customization",
        status: Compatibility::ClientSideCandidate,
        evidence: "AyuGram feature list; local presentation behavior",
    },
    FeatureNote {
        name: "Streamer Mode",
        status: Compatibility::ResearchRequired,
        evidence: "AyuGram describes capture hiding; Wayland compositor/API feasibility unverified",
    },
];
