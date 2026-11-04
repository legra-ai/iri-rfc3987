//! Scheme/path state and shared RFC character tables.

use super::authority::Authority;
use crate::pct_enc::Encoder;
use crate::pct_enc::encoder::{IFragment, IPath, IQuery, ISegmentNzNc};

#[derive(Debug, Default)]
enum Phase {
    #[default]
    Start,
    FirstSegment {
        scheme: bool,
    },
    AfterScheme,
    Slash,
    Authority(Authority),
    Path,
    Query,
    Fragment,
}

#[derive(Debug, Default)]
pub(super) struct Grammar {
    phase: Phase,
    pub(super) absolute: bool,
}

pub(super) fn allows<E: Encoder>(scalar: char, encoded: bool) -> bool {
    if encoded {
        E::TABLE.allows_pct_encoded()
    } else {
        E::TABLE.allows_code_point(u32::from(scalar))
    }
}

impl Grammar {
    pub(super) fn atom(&mut self, scalar: char, encoded: bool) -> bool {
        match &mut self.phase {
            Phase::Start => {
                if !encoded && scalar == '/' {
                    self.phase = Phase::Slash;
                    true
                } else if !encoded && matches!(scalar, '?' | '#') {
                    self.delimiter(scalar)
                } else {
                    self.phase = Phase::FirstSegment {
                        scheme: !encoded && scalar.is_ascii_alphabetic(),
                    };
                    allows::<ISegmentNzNc>(scalar, encoded)
                }
            }
            Phase::FirstSegment { scheme } => {
                if !encoded && scalar == ':' {
                    if !*scheme {
                        return false;
                    }
                    self.absolute = true;
                    self.phase = Phase::AfterScheme;
                    true
                } else if !encoded && matches!(scalar, '/' | '?' | '#') {
                    self.delimiter(scalar)
                } else {
                    *scheme &= !encoded
                        && (scalar.is_ascii_alphanumeric() || matches!(scalar, '+' | '-' | '.'));
                    allows::<ISegmentNzNc>(scalar, encoded)
                }
            }
            Phase::AfterScheme => {
                if !encoded && scalar == '/' {
                    self.phase = Phase::Slash;
                    true
                } else {
                    self.phase = Phase::Path;
                    self.atom(scalar, encoded)
                }
            }
            Phase::Slash => {
                if !encoded && scalar == '/' {
                    self.phase = Phase::Authority(Authority::default());
                    true
                } else {
                    self.phase = Phase::Path;
                    self.atom(scalar, encoded)
                }
            }
            Phase::Authority(authority) => {
                if !encoded && matches!(scalar, '/' | '?' | '#') {
                    authority.finish() && self.delimiter(scalar)
                } else {
                    authority.atom(scalar, encoded)
                }
            }
            Phase::Path => {
                if !encoded && matches!(scalar, '?' | '#') {
                    self.delimiter(scalar)
                } else {
                    allows::<IPath>(scalar, encoded)
                }
            }
            Phase::Query => {
                if !encoded && scalar == '#' {
                    self.delimiter(scalar)
                } else {
                    allows::<IQuery>(scalar, encoded)
                }
            }
            Phase::Fragment => allows::<IFragment>(scalar, encoded),
        }
    }

    fn delimiter(&mut self, scalar: char) -> bool {
        self.phase = match scalar {
            '/' => Phase::Path,
            '?' => Phase::Query,
            '#' => Phase::Fragment,
            _ => unreachable!("grammar delimiter"),
        };
        true
    }

    pub(super) fn finish(&self) -> bool {
        match &self.phase {
            Phase::Authority(authority) => authority.finish(),
            _ => true,
        }
    }
}
