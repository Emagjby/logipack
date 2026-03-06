use crate::errors::TransitionError;
use crate::shipment::ShipmentStatus;

pub fn validate_transition(
    from: ShipmentStatus,
    to: ShipmentStatus,
    office_changed: bool,
    has_current_office: bool,
    has_target_office: bool,
) -> Result<(), TransitionError> {
    // Terminal states reject all
    if from.is_terminal() {
        return Err(TransitionError::TerminalState { from });
    }

    // Office hop policy
    if office_changed
        && !(to == ShipmentStatus::InTransit
            || (from == ShipmentStatus::InTransit && to == ShipmentStatus::Accepted))
    {
        return Err(TransitionError::OfficeHopNotAllowed { from, to });
    }

    if office_changed && !has_current_office {
        return Err(TransitionError::OfficeHopRequiresCurrentOffice { to });
    }

    if !has_current_office
        && !has_target_office
        && matches!(to, ShipmentStatus::Accepted | ShipmentStatus::Processed)
    {
        return Err(TransitionError::OfficeRequiredForStatus { to });
    }

    use ShipmentStatus::*;

    let allowed = matches!(
        (from, to),
        // forward progression
        (New, Accepted)
            | (Accepted, Processed)
            | (Processed, InTransit)
            | (InTransit, Accepted)
            | (InTransit, Delivered)
            // cancellation
            | (New, Cancelled)
            | (Accepted, Cancelled)
            | (Processed, Cancelled)
            | (InTransit, Cancelled)
    );

    if allowed {
        Ok(())
    } else {
        Err(TransitionError::InvalidTransition { from, to })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ShipmentStatus::*;

    #[test]
    fn allowed_forward_transitions_pass() {
        let cases = [
            (New, Accepted),
            (Accepted, Processed),
            (Processed, InTransit),
            (InTransit, Delivered),
        ];

        for (from, to) in cases {
            assert!(
                validate_transition(from, to, false, true, false).is_ok(),
                "expected {:?} -> {:?} to be allowed",
                from,
                to
            )
        }
    }

    #[test]
    fn cancellation_is_allowed_from_non_terminal_states() {
        let cases = [New, Accepted, Processed, InTransit];

        for from in cases {
            assert!(
                validate_transition(from, Cancelled, false, true, false).is_ok(),
                "expected {:?} -> Cancelled to be allowed",
                from
            )
        }
    }

    #[test]
    fn terminal_states_reject_all_transitions() {
        let cases = [Delivered, Cancelled];

        for from in cases {
            let err = validate_transition(from, New, false, true, false).unwrap_err();
            assert!(
                matches!(err, TransitionError::TerminalState { .. }),
                "expected terminal state error for {:?}",
                from
            );
        }
    }

    #[test]
    fn invalid_transitions_are_rejected() {
        let cases = [
            (New, Processed),
            (Accepted, InTransit),
            (Processed, Delivered),
            (New, Delivered),
        ];

        for (from, to) in cases {
            let err = validate_transition(from, to, false, true, false).unwrap_err();
            assert!(
                matches!(err, TransitionError::InvalidTransition { .. }),
                "expected invalid transition {:?} -> {:?}",
                from,
                to
            );
        }
    }

    #[test]
    fn office_hop_is_only_allowed_when_transitioning_to_in_transit() {
        // allowed:
        assert!(
            validate_transition(Processed, InTransit, true, true, true).is_ok(),
            "office hop should be allowed when going to IN_TRANSIT"
        );

        // disallowed:
        let err = validate_transition(New, Accepted, true, true, true).unwrap_err();
        assert!(
            matches!(err, TransitionError::OfficeHopNotAllowed { .. }),
            "office hop should be rejected outside IN_TRANSIT"
        )
    }

    #[test]
    fn office_hop_requires_current_office() {
        let err = validate_transition(Processed, InTransit, true, false, true).unwrap_err();
        assert!(
            matches!(err, TransitionError::OfficeHopRequiresCurrentOffice { .. }),
            "office hop should require current office"
        );
    }

    #[test]
    fn office_hop_is_allowed_when_arriving_to_next_office() {
        assert!(
            validate_transition(InTransit, Accepted, true, true, true).is_ok(),
            "office hop should be allowed when moving from IN_TRANSIT to ACCEPTED"
        );
    }

    #[test]
    fn accepted_requires_office_when_current_unknown() {
        let err = validate_transition(New, Accepted, false, false, false).unwrap_err();
        assert!(
            matches!(err, TransitionError::OfficeRequiredForStatus { .. }),
            "accepted should require an office when current office is unknown"
        );
    }
}
