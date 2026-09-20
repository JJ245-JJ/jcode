//! The operator override, checked at both seams.
//!
//! Lives in its own integration-test binary on purpose: `JCODE_DISABLE_RISK_GATE`
//! is process-global, so setting it inside the unit-test binary would race the
//! other tests running in parallel. One test, one process, no race.

use jcode_command_risk::{
    GateOutcome, Justification, RiskContext, assess, gate, is_catastrophic_target,
};
use std::path::{Path, PathBuf};

#[test]
fn disable_env_var_opens_both_seams_and_restores_when_unset() {
    let ctx = RiskContext {
        working_dir: Some(PathBuf::from("/home/u/proj")),
        home_dir: Some(PathBuf::from("/home/u")),
    };
    let no_justification = Justification::default();

    // A command that normally reflects, and a path that is normally an
    // absolute deny. These are the two independent code paths.
    let reflects = assess("rm -rf $TARGET", &ctx);
    let catastrophic = Path::new("/etc/passwd");

    // Baseline: the gate is on unless explicitly disabled.
    unsafe { std::env::remove_var("JCODE_DISABLE_RISK_GATE") };
    assert!(
        matches!(
            gate(&reflects, &no_justification),
            GateOutcome::Reflect { .. }
        ),
        "gate should hold an unresolvable rm by default"
    );
    assert!(
        is_catastrophic_target(catastrophic, &ctx),
        "/etc/passwd should be protected by default"
    );

    // Disabled: both seams open.
    unsafe { std::env::set_var("JCODE_DISABLE_RISK_GATE", "1") };
    assert!(
        matches!(gate(&reflects, &no_justification), GateOutcome::Allow),
        "JCODE_DISABLE_RISK_GATE=1 must allow a command the gate would hold"
    );
    assert!(
        !is_catastrophic_target(catastrophic, &ctx),
        "JCODE_DISABLE_RISK_GATE=1 must also open the apply_patch seam"
    );

    // Unsetting restores: this is reversible without a rebuild.
    unsafe { std::env::remove_var("JCODE_DISABLE_RISK_GATE") };
    assert!(
        matches!(
            gate(&reflects, &no_justification),
            GateOutcome::Reflect { .. }
        ),
        "unsetting the var must restore the gate"
    );
    assert!(
        is_catastrophic_target(catastrophic, &ctx),
        "unsetting the var must restore the catastrophic deny"
    );
}
