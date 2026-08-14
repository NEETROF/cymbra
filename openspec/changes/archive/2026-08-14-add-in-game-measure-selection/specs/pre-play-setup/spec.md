## REMOVED Requirements

### Requirement: Full-run vs selective-practice choice in setup

**Reason**: The deliberate range-selection surface moves into the game screen (the
dedicated measure-selection mode of `measure-range-practice`); keeping a second
surface in the pre-play modal would duplicate the same choice in a place players
found impractical. The setup modal becomes range-neutral: it never sets or clears
the active range, and an armed range survives opening and dismissing it.

**Migration**: Pick or clear a range in-game — long-press the transport
measure-rewind control to open the selection mode (confirm applies, whole-piece
clears). Per-score saved practice settings now pre-fill the selection mode's draft
instead of the modal picker. The end-of-run summary's "practice this section"
picker is unaffected.
